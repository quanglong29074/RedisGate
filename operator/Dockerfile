# ---- Build Stage ----
FROM rust:1.75-alpine AS builder

# Install build dependencies for musl target
RUN apk add --no-cache build-base musl-dev

WORKDIR /app

# Build for MUSL (static-ish) so it runs on Alpine
RUN rustup target add x86_64-unknown-linux-musl

# Copy operator source and build release binary
# Note: expects a Cargo project in this directory in the future
COPY . .
RUN cargo build --release --bin operator --target x86_64-unknown-linux-musl \
    && strip target/x86_64-unknown-linux-musl/release/operator || true


# ---- Runtime Stage ----
FROM alpine:3.20

# Create non-root user
RUN adduser -D -H -u 10001 appuser

# Copy the built binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/operator /operator

EXPOSE 8080

# Simple healthcheck: ensures the main process exists
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD /bin/sh -c 'pidof operator >/dev/null 2>&1 || exit 1'

USER appuser
ENTRYPOINT ["/operator"]
