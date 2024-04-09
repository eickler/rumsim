FROM rust:slim as builder
WORKDIR /app
COPY . /app
RUN cargo test --release
RUN cargo build --release

FROM gcr.io/distroless/cc-debian12
COPY --from=builder /app/target/release/rumsim /
CMD ["./rumsim"]
