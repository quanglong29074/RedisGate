# Multi-stage build for RedisGate
FROM rustlang/rust:nightly as builder

# Install build dependencies including Bun
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    unzip \
    && curl -fsSL https://bun.sh/install | bash \
    && rm -rf /var/lib/apt/lists/*

# Add Bun to PATH
ENV PATH="/root/.bun/bin:$PATH"

WORKDIR /app

# Accept DATABASE_URL as build argument
ARG DATABASE_URL

# Copy build files
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations

# Copy and setup frontend
COPY app/frontend-redis ./app/frontend-redis

# Install frontend dependencies and build
RUN cd app/frontend-redis && \
    bun install && \
    bun run build

# Install PostgreSQL for build-time database (needed for SQLX macros)
RUN apt-get update && apt-get install -y postgresql postgresql-contrib && \
    service postgresql start && \
    su - postgres -c "createuser --superuser redisgate" && \
    su - postgres -c "createdb redisgate_dev" && \
    su - postgres -c "psql -c \"ALTER USER redisgate PASSWORD 'redisgate_dev_password';\""

# Check PostgreSQL status and run migrations before building
RUN echo "Setting up database for SQLX macros" && \
    service postgresql start && \
    sleep 10 && \
    su - postgres -c "psql -c \"\\l\"" && \
    su - postgres -c "psql -c \"SELECT version();\"" && \
    DATABASE_URL="postgresql://redisgate:redisgate_dev_password@localhost:5432/redisgate_dev" cargo install sqlx-cli@0.7.4 --no-default-features --features native-tls,postgres && \
    DATABASE_URL="postgresql://redisgate:redisgate_dev_password@localhost:5432/redisgate_dev" sqlx migrate run && \
    DATABASE_URL="postgresql://redisgate:redisgate_dev_password@localhost:5432/redisgate_dev" cargo build --release --bin redisgate --verbose

# Runtime stage - minimal image
FROM rustlang/rust:nightly

# Install runtime dependencies
RUN apt-get update && \
    apt-get install -y ca-certificates curl libssl3 && \
    rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -r -s /bin/false -m redisgate

# Set up working directory
RUN mkdir -p /app && chown redisgate:redisgate /app
WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/redisgate /usr/local/bin/redisgate

# Copy migrations
COPY --from=builder /app/migrations ./migrations

# Copy built frontend static files
COPY --from=builder /app/app/frontend-redis/dist ./app/frontend-redis/dist

# Copy startup script (optional)
COPY start.sh ./start.sh
RUN chmod +x ./start.sh

# Change ownership to app user
RUN chown -R redisgate:redisgate /app /usr/local/bin/redisgate

USER redisgate

# Expose only port 8080 (unified backend + frontend)
EXPOSE 8080

# Set default environment variables
ENV RUST_LOG=info
ENV APP_HOST=0.0.0.0
ENV APP_PORT=8080

# Note: DATABASE_URL should be provided when running the container
# Example: docker run -e DATABASE_URL="postgresql://user:pass@host:5432/db" redisgate:latest

# Health check - only check backend since it serves everything
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Use start script or run binary directly
CMD ["./start.sh"]
# Alternative: CMD ["/usr/local/bin/redisgate"]