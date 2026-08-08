FROM rust:1-slim AS builder
RUN apt-get update && apt-get install -y --no-install-recommends pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY mediavault-api/Cargo.toml mediavault-api/Cargo.toml
COPY api-client-lib/Cargo.toml api-client-lib/Cargo.toml
COPY mediavault-mcp/Cargo.toml mediavault-mcp/Cargo.toml
RUN mkdir -p mediavault-api/src api-client-lib/src mediavault-mcp/src \
    && echo "fn main() {}" > mediavault-api/src/main.rs \
    && echo "" > api-client-lib/src/lib.rs \
    && echo "fn main() {}" > mediavault-mcp/src/main.rs \
    && cargo build --release -p mediavault-mcp \
    && rm -rf mediavault-api/src api-client-lib/src mediavault-mcp/src
COPY . .
RUN touch mediavault-mcp/src/main.rs \
    && cargo build --release -p mediavault-mcp

FROM debian:bookworm-slim AS runtime
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/mediavault-mcp /app/mediavault-mcp
EXPOSE 8081
ENTRYPOINT ["/app/mediavault-mcp"]
