## ---- Build Stage ----
FROM rust:1.75-alpine AS builder

# Install build dependencies for musl target (for small static-ish binary)
RUN apk add --no-cache build-base musl-dev

WORKDIR /app

# Build for MUSL so the binary runs on Alpine/distroless
RUN rustup target add x86_64-unknown-linux-musl

# Copy source and build optimized release binary
COPY . .
RUN cargo build --release --bin gateway --target x86_64-unknown-linux-musl \
    && strip target/x86_64-unknown-linux-musl/release/gateway


## ---- Runtime Stage ----
FROM alpine:3.20

# Create non-root user for security
RUN adduser -D -H -u 10001 appuser

# Copy the built binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/gateway /gateway

EXPOSE 8080

# Healthcheck prefers /healthz, falls back to /
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD sh -c 'wget -q --spider http://127.0.0.1:8080/healthz || wget -q --spider http://127.0.0.1:8080/ || exit 1'

USER appuser
ENTRYPOINT ["/gateway"]
