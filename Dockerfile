# Build stage
FROM rust:1.83-slim AS builder

ARG HTTP_PROXY HTTPS_PROXY NO_PROXY
ENV HTTP_PROXY=${HTTP_PROXY} HTTPS_PROXY=${HTTPS_PROXY} NO_PROXY=${NO_PROXY}

WORKDIR /app

RUN apt-get update && apt-get install -y pkg-config libssl-dev protobuf-compiler && rm -rf /var/lib/apt/lists/*

COPY Cargo.toml Cargo.lock build.rs ./
COPY src/ ./src/
COPY protos/ ./protos/

RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y procps chromium ca-certificates && rm -rf /var/lib/apt/lists/*

ENV CHROME_BIN=/usr/bin/chromium CHROME_PATH=/usr/bin/chromium

WORKDIR /app

COPY --from=builder /app/target/release/chaser-oxide-server /app/
COPY --from=builder /app/protos /app/protos
RUN mkdir -p /app/data

EXPOSE 50051

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pgrep -f chaser-oxide-server || exit 1

CMD ["./chaser-oxide-server"]
