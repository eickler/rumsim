# Running the benchmark

## Configure an MQTT server to send data to

```
export URL=<mqtt://mqtt.myserver.io:1883>
export USER=<user>
export PASS=<pass>
export CLIENT_ID=<client ID>
export QOS=0
```

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

This is the current format that data is sent in:

Topic: /device\_{cluster ID}\_{device ID}/{data point name}
Format time,value

Cluster ID distinguishes devices from several running simulators.

## TBDs

- C8Y format support:
  Topic is s/us/<device ID>
  Payload could be per device
  201,S,<time>,SF,<data point 1>,<value 1>,<unit>,<data point 2>,<value 2>,<unit>,...
  or
  200,SF,<data point 1>,<value 1>
  200,SF,<data point 2>,<value 2>
  ...
  Which one? I guess the first one creates only one transaction and will be faster despite of the additional text...also simplifies the simulator.

- Observability support using OTLP and Grafana
- Liveness/readiness probes?
- Compile into a static image with libmusl and try from:scratch container.
- Implement an operator to distribute and scale the workload? Maybe even auto-scale?
