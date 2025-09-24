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

# Copy and setup frontend for development
COPY app/frontend-redis ./app/frontend-redis
RUN cd app/frontend-redis && \ 
bun install

# Build the application
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

# Runtime stage - use the same Debian version as the builder
FROM rustlang/rust:nightly

# Install runtime dependencies including Bun
RUN apt-get update && \
    apt-get install -y ca-certificates curl libssl3 unzip \
    && curl -fsSL https://bun.sh/install | bash \
    && rm -rf /var/lib/apt/lists/*

# Add Bun to PATH for root user
ENV PATH="/root/.bun/bin:$PATH"

# Create non-root user for security
RUN useradd -r -s /bin/false -m redisgate

# Set up working directory first
RUN mkdir -p /app && chown redisgate:redisgate /app
WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/redisgate /usr/local/bin/redisgate

# Copy migrations
COPY --from=builder /app/migrations ./migrations

# Copy frontend
COPY --from=builder /app/app/frontend-redis ./frontend-redis

# Copy startup script to the working directory
COPY start.sh ./start.sh
RUN chmod +x ./start.sh

# Copy Bun to system location and make it accessible to redisgate user
RUN cp /root/.bun/bin/bun /usr/local/bin/bun && \
    chmod +x /usr/local/bin/bun

# Change ownership to app user (including start.sh)
RUN chown -R redisgate:redisgate /app /usr/local/bin/redisgate
USER redisgate

# Expose ports: 8080 for backend, 3000 for frontend dev server
EXPOSE 8080 3000

# Set default environment variables
ENV RUST_LOG=info
ENV APP_HOST=0.0.0.0
ENV APP_PORT=8080

# Note: DATABASE_URL should be provided when running the container
# Example: docker run -e DATABASE_URL="postgresql://user:pass@host:5432/db" redisgate:latest

# Health check - check both backend health and frontend availability in development
HEALTHCHECK --interval=30s --timeout=10s --start-period=30s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

CMD ["./start.sh"]