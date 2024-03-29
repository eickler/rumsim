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

/*
If setting the global meter provider does not work, we can change this to:
pub fn new_meter() -> OtlpMetricPipeline<Tokio, MetricsExporterBuilder> {
    opentelemetry_otlp::new_pipeline()
        .metrics(opentelemetry_sdk::runtime::Tokio)
        .with_exporter(new_exporter())
*/

pub struct Metering {
    datapoint_sec: Gauge<f64>,
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

        let unit = Unit::new("1/s");
        let datapoint_sec = meter.f64_gauge("datapoints").with_unit(unit).init();
        let overload_cnt = meter.f64_counter("overload").init();

        Metering {
            datapoint_sec,
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
}
