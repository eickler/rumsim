# rumsim: A data generator for simulation and benchmarking IoT workloads.

## Quickstart

For scalable workload generation, please see the associated [Kubernetes operator](https://github.com/eickler/rumsimop).

You can directly run the simulator by setting a bunch of environment variables.

### Broker-related variables

| Variable         | Default               | Description                                           |
| ---------------- | --------------------- | ----------------------------------------------------- |
| BROKER_URL       | mqtt://localhost:1883 | The MQTT broker to send data to.                      |
| BROKER_USER      | mqtt                  | The username for connecting to the broker.            |
| BROKER_PASS      | pass                  | The password for connecting to the broker.            |
| BROKER_CLIENT_ID | rumsim-0              | The client ID for connecting to the broker.           |
| BROKER_QOS       | 1                     | The quality of service (0..2) used for MQTT messages. |

### Simulation-related variables

| Variable           | Default       | Description                                        |
| ------------------ | ------------- | -------------------------------------------------- |
| SIM_DEVICES        | 100           | The number of devices to simulate.                 |
| SIM_DATA_POINTS    | 100           | The number of data points per devices to simulate. |
| SIM_SEED           | 0             | The random number seed for generating data.        |
| SIM_FREQUENCY_SECS | 1             | How often the data should be generated.            |
| SIM_START_TIME     | \<immediate\> | ISO datetime when the simulator starts generating. |
| SIM_RUNS           | usize::MAX    | Number of simulator runs.                          |

### Observability-related variables

| Variable      | Default     | Description                                   |
| ------------- | ----------- | --------------------------------------------- |
| OTLP_ENDPOINT | \<console\> | URL of OTLP collector for traces and metrics. |
| OTLP_AUTH     | \<unset\>   | Authentication string for OTLP collector.     |

### Other configuration

| Variable | Default | Description                 |
| -------- | ------- | --------------------------- |
| CAPACITY | 1000    | Capacity of message buffer. |
| RUST_LOG | info    | OTLP trace level.           |

Trace levels are:

- trace: Individual data points that are generated.
- debug: Start and stop of individual simulation runs.
- info: Start and stop of simulation.
- error: Error messages.

## Build and run the device simulator

```
cargo build -r
# Start your MQTT broker.
# Set environment variables (or try the defaults).
target/release/rumsim
```

## Message format

Data is sent in [Cumulocity IoT SmartREST 2.0 format](https://cumulocity.com/docs/smartrest/smartrest-two/).

Topic:

```
s/us/{BROKER_CLIENT_ID}\_{device ID}
```

Payload:

```
201,S,<time>,SF,<datapoint 1>,<value 1>,,SF,<datapoint 2>,<value 2>,…
```

Notes:

- BROKER_CLIENT_ID should be different for each instance of the simulator. Using the Kubernetes operator, the BROKER_CLIENT_ID is the ID of the pod (name of the simulation plus a running number).
- The device ID is a running number.

## Known issues

The simulator currently just crashes if you send so many data points that the maximum packet size of the MQTT broker is exceeded:

```
2024-04-02T15:09:50.377381Z WARN rumsim: Failed to connect error=MqttState(OutgoingPacketTooLarge { pkt_size: 18136, max: 10240 })
```

However, the last messages are apparently not correctly forwarded to the OTLP endpoint for some reason (even though I call the shutdown method), so you currently do not see why the simulator crashed.

## Notes/ideas

- Add more unit tests.
- Try passing opentelemetry span IDs through MQTT 5? Is it possible to have an MQTT 3 fallback for servers not supporting mqtt 5?
- Make OpenTelemetry and Tonic dependencies optional, put observability into an optional module and have a feature flag to compile OTLP support in or not. It looks like the whole observability stack adds 5 MB to the final binary?
