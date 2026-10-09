# ==========================================
# Multi-Stage Build: MikroTik Universal Rust Engine
# ==========================================
# Stage 1: Build binary using official Rust Alpine image (musl static)
FROM rust:alpine AS builder

WORKDIR /app

# Install build dependencies for Alpine/musl
RUN apk add --no-cache musl-dev

# Copy manifests first to optimize Docker layer caching
COPY Cargo.toml Cargo.lock ./
COPY crates/routeros-core/Cargo.toml ./crates/routeros-core/
COPY crates/routeros-gateway/Cargo.toml ./crates/routeros-gateway/

# Create dummy source files to cache dependency compilation
RUN mkdir -p ./crates/routeros-core/src && echo "pub fn dummy() {}" > ./crates/routeros-core/src/lib.rs && \
    mkdir -p ./crates/routeros-gateway/src && echo "fn main() {}" > ./crates/routeros-gateway/src/main.rs

RUN cargo build --release -p routeros-gateway || true

# Copy real source code
COPY crates/routeros-core ./crates/routeros-core
COPY crates/routeros-gateway ./crates/routeros-gateway

# Touch main files to invalidate cargo cache for real code build
RUN touch ./crates/routeros-core/src/lib.rs ./crates/routeros-gateway/src/main.rs

# Build final release binary
RUN cargo build --release -p routeros-gateway

# ==========================================
# Stage 2: Ultra-lightweight & Secure Production Runtime (< 25MB)
# ==========================================
FROM alpine:3.20 AS runtime

RUN apk add --no-cache ca-certificates tzdata wget

# Create non-root user for security
RUN addgroup -S appgroup && adduser -S appuser -G appgroup

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/routeros-gateway /usr/local/bin/routeros-gateway

# Copy default config and set permissions
COPY config.example.toml /app/config.toml
RUN chown -R appuser:appgroup /app

USER appuser

EXPOSE 8080

ENV GATEWAY_LISTEN="0.0.0.0:8080" \
    GATEWAY_TOKEN="change-me-to-a-long-random-string" \
    RUST_LOG="info"

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget -qO- http://127.0.0.1:8080/health || exit 1

ENTRYPOINT ["routeros-gateway"]
CMD ["/app/config.toml"]
