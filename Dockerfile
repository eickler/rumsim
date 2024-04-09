FROM rust:slim as builder
WORKDIR /app
COPY . /app
RUN cargo test --release
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
LABEL org.opencontainers.image.title="rumsim" \
  org.opencontainers.image.description="A data generator for simulation and benchmarking IoT workloads." \
  org.opencontainers.image.source="https://github.com/eickler/rumsim"
COPY --from=builder /app/target/release/rumsim /
CMD ["./rumsim"]
