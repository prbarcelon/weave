FROM rust:1.85-slim AS builder

RUN apt-get update && apt-get install -y pkg-config libssl-dev && rm -rf /var/lib/apt/lists/*
RUN rustup update 1.93.0 && rustup default 1.93.0

WORKDIR /app
COPY . .

RUN cargo build --release -p weave-github

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/weave-github /usr/local/bin/weave-github

EXPOSE 8080
CMD ["weave-github"]
