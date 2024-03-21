# Build time
FROM rust:slim as builder

WORKDIR /usr/src/rumsim
COPY . .
RUN cargo build --release

# Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/src/rumsim/target/release/rumsim /usr/local/bin/rumsim
COPY defaults.toml .

CMD ["rumsim"]
