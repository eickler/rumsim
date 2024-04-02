# Running the benchmark

## Configure an MQTT server to send data to

```
export URL=<mqtt://mqtt.myserver.io:1883>
export USER=<user>
export PASS=<pass>
export CLIENT_ID=<client ID>
export QOS=0
```

## Optionally configure an OTLP endpoint to send traces and metrics to

```
export OTLP_COLLECTOR=https://localhost:4317
export OTLP_AUTH=...
export RUST_LOG=...
```

Log levels are:

- trace: Individual data points that are generated.
- debug: Start and stop of individual simulation runs.
- info: Start and stop of simulation.
- error: Error messages.

## Build and run the device simulator

```
cargo build
target/debug/rumsim
```

## Control the device simulator through MQTT

- Connect an client of your choice to the broker.
- Send commands to the topic "control" (or your topic configured with the variable CONTROL_TOPIC).
- Starting the simulation:

```
start <devices> <data points> <wait time in secs> <seed>
start 2 2 5 1
```

- Stopping the simulation:

```
stop
```

## Message format

Data is sent in Cumulocity SmartREST 2.0 format.

Topic:

```
s/us/{instance ID}\_{device ID}
```

Payload:

```
201,S,<time>,SF,<datapoint 1>,<value 1>,,SF,<datapoint 2>,<value 2>,…
```

"instance ID" is the ID of the simulator POD in Kubernetes in case of multiple PODs, otherwise it's the configured client ID.

## Known issues

The simulator currently just crashes if you send so many data points that the maximum packet size of the MQTT broker is exceeded:

```
2024-04-02T15:09:50.377381Z WARN rumsim: Failed to connect error=MqttState(OutgoingPacketTooLarge { pkt_size: 18136, max: 10240 })
```

However, the last messages are apparently not correctly forwarded to the OTLP endpoint for some reason (even though I call the shutdown method).

## Ideas

- Implement an operator to distribute and scale the workload? Maybe even auto-scale?
- Set "deployment.environment" for traces to show up in Aspecto (Kubernetes? Which cluster?)
- Compile into a static image with libmusl and try from:scratch container.
