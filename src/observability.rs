use opentelemetry::{
    global::{self},
    metrics::{Counter, Gauge, Unit},
    Key, KeyValue,
};
use opentelemetry_otlp::{TonicExporterBuilder, WithExportConfig};
use opentelemetry_sdk::{trace as sdktrace, Resource};
use opentelemetry_semantic_conventions::resource::SERVICE_NAME;
use tokio::time::Duration;
use tonic::metadata::MetadataMap;
use tracing_subscriber::{prelude::*, EnvFilter};

use crate::CONFIG;

fn new_exporter() -> TonicExporterBuilder {
    let mut map = MetadataMap::with_capacity(1);
    if let Some(auth) = &CONFIG.otlp_auth {
        map.insert("authorization", auth.parse().unwrap());
    }
    opentelemetry_otlp::new_exporter()
        .tonic()
        .with_endpoint(CONFIG.otlp_collector.clone())
        .with_timeout(Duration::from_secs(3))
        .with_metadata(map.clone())
}

pub fn init_tracing() {
    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(new_exporter())
        .with_trace_config(
            sdktrace::config()
                .with_resource(Resource::new(vec![KeyValue::new(SERVICE_NAME, "rumsim")])),
        )
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .expect("Failed to initialize tracer.");

    let subscriber = tracing_subscriber::registry()
        .with(EnvFilter::from_default_env())
        .with(tracing_opentelemetry::layer().with_tracer(tracer));

    tracing::subscriber::set_global_default(subscriber).unwrap();
}

pub fn init_metering() {
    let meter_provider = opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(new_exporter())
        .build()
        .unwrap();
    global::set_meter_provider(meter_provider);
}

pub struct Metering {
    datapoint_sec: Gauge<f64>,
    capacity_percent: Gauge<f64>,
    overload_cnt: Counter<f64>,
    labels: Vec<KeyValue>,
}

impl Metering {
    pub fn new() -> Metering {
        init_metering();

        let meter = global::meter("rumsim");

        let labels = vec![
            Key::new(SERVICE_NAME).string("rumsim"),
            Key::new("service.replica").string(CONFIG.client_id.clone()),
        ];

        let dp_unit = Unit::new("1/s");
        let datapoint_sec = meter.f64_gauge("datapoints").with_unit(dp_unit).init();

        let cap_unit = Unit::new("%");
        let capacity_percent = meter.f64_gauge("capacity").with_unit(cap_unit).init();

        let overload_cnt = meter.f64_counter("overload").init();

        Metering {
            datapoint_sec,
            capacity_percent,
            overload_cnt,
            labels,
        }
    }

    pub fn is_overloaded(&self) {
        self.overload_cnt.add(1.0, &self.labels);
    }

    pub fn record_datapoints(&self, datapoints: usize, elapsed: Duration) {
        let dpsec_value = datapoints as f64 / elapsed.as_secs_f64();
        self.datapoint_sec.record(dpsec_value, &self.labels);
    }

    pub fn record_capacity(&self, elapsed: Duration, wait_time: Duration) {
        let cap_value = elapsed.as_secs_f64() / wait_time.as_secs_f64() * 100.0;
        self.capacity_percent.record(cap_value, &self.labels);
    }
}
