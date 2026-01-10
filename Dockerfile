# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY build.rs ./
COPY src/ ./src/
COPY protos/ ./protos/

# Build release binary
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    chromium \
    chromium-driver \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Set up chromium for headless operation
ENV CHROME_BIN=/usr/bin/chromium
ENV CHROME_PATH=/usr/bin/chromium

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/chaser-oxide-server /app/chaser-oxide-server

# Copy proto files if needed
COPY --from=builder /app/protos /app/protos

# Create data directory
RUN mkdir -p /app/data

# Expose gRPC port
EXPOSE 50051

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD pgrep -f chaser-oxide-server || exit 1

# Run the server
CMD ["./chaser-oxide-server"]
