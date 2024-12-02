# rust builder
FROM rust:latest AS builder

WORKDIR /build

COPY . .

RUN cargo build --release

# runner container
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl-dev && apt-get clean

ENV RUST_LOG=info

WORKDIR /runner

COPY --from=builder /build/target/release/raline-server ./app

COPY ./config ./config

COPY ./data ./data

EXPOSE 8080

ENTRYPOINT ["/runner/app"]