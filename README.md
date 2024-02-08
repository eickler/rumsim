# Running the benchmark

## Install and start the nats-server

```
brew install nats-server
nats-server -c server.conf
```

## Build and run the device simulator

```
cargo build
target/debug/rumsim
```

## Control the device simulator from nats

```
brew install nats-io/nats-tools/nats
```

Start the simulated devices:

```
nats --user=mqtt --password=pass pub control "start <devices> <data points> <wait time in secs> <seed>"
nats --user=mqtt --password=pass pub control "start 2 2 5 1"
```

You can send a start command at any time to change the simulated devices. Stop the simulated devices:

```
nats --user=mqtt --password=pass pub control "stop"
```

## Message format

Topic: /device\_{cluster ID}\_{device ID}/{data point name}
Format time,value

Cluster ID distinguishes devices from several running simulators.
