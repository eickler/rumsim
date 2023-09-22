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
nats --user=mqtt --password=pass pub control "start <devices> <data points> <wait time in secs>"
nats --user=mqtt --password=pass pub control "start 2 2 5"
```

You can send a start command at any time to change the simulated devices. Stop the simulated devices:

```
nats --user=mqtt --password=pass pub control "stop"
```
