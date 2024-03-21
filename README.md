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
