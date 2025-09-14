# RedisGate Development Environment

This directory contains scripts and configuration files to set up a complete development environment for the RedisGate project.

## Quick Start

**For Development (Simple Workflow):**
```bash
# 1. One-time setup - installs dependencies and starts services
./setup-dev.sh

# 2. Build the application 
cargo build

# 3. Run the application (migrations run automatically)
cargo run
```

**For Manual Step-by-Step Setup:**

1. **One-time setup** (installs all dependencies):
   ```bash
   ./setup-dev.sh
   ```

2. **Start development services**:
   ```bash
   ./scripts/dev-services.sh start
   ```

3. **Start Minikube for Kubernetes development**:
   ```bash
   ./scripts/minikube-dev.sh start
   ```

## Files Overview

### Configuration Files

- **`env.development`** - Environment variables for development
- **`docker-compose.yml`** - PostgreSQL service configuration
- **`migrations/`** - Database migration files (replaces deprecated init-db.sql)

### Scripts

- **`setup-dev.sh`** - Main setup script for development environment
- **`scripts/dev-services.sh`** - Manage external services (PostgreSQL)
- **`scripts/minikube-dev.sh`** - Manage Minikube cluster

## Detailed Setup

### 1. Main Setup Script (`setup-dev.sh`)

The main setup script installs and configures all development dependencies:

**Full setup:**
```bash
./setup-dev.sh
```

**Options:**
```bash
./setup-dev.sh --help                # Show help
./setup-dev.sh --docker-only         # Setup only Docker and external services
./setup-dev.sh --rust-only           # Setup only Rust toolchain
./setup-dev.sh --k8s-only            # Setup only Kubernetes tools
./setup-dev.sh --no-minikube         # Skip Minikube setup
./setup-dev.sh --no-services         # Skip starting services
```

**What it installs:**
- **Docker & Docker Compose** - For containerized services
- **Rust** - Programming language and toolchain
- **kubectl** - Kubernetes command-line tool
- **Minikube** - Local Kubernetes cluster
- **PostgreSQL** - External dependency via Docker

### 2. Services Management (`scripts/dev-services.sh`)

Manage PostgreSQL service:

```bash
# Start all services
./scripts/dev-services.sh start

# Stop all services
./scripts/dev-services.sh stop

# Show services status
./scripts/dev-services.sh status

# View logs (all services)
./scripts/dev-services.sh logs

# View logs (specific service)
./scripts/dev-services.sh logs postgres

# Connect to PostgreSQL
./scripts/dev-services.sh psql

# Clean restart (removes all data)
./scripts/dev-services.sh clean

# Reset database only
./scripts/dev-services.sh reset-db
```

### 3. Minikube Management (`scripts/minikube-dev.sh`)

Manage local Kubernetes cluster:

```bash
# Start Minikube cluster
./scripts/minikube-dev.sh start

# Stop Minikube cluster
./scripts/minikube-dev.sh stop

# Show cluster status
./scripts/minikube-dev.sh status

# Open Kubernetes dashboard
./scripts/minikube-dev.sh dashboard

# Create development namespace
./scripts/minikube-dev.sh namespace

# Enable local Docker registry
./scripts/minikube-dev.sh registry

# Load local images to Minikube
./scripts/minikube-dev.sh load-images
```

## Configuration Details

### Environment Variables (`env.development`)

Key configuration variables:

```bash
# PostgreSQL
POSTGRES_HOST=localhost
POSTGRES_PORT=5432
POSTGRES_USER=redisgate_dev
POSTGRES_PASSWORD=redisgate_dev_password
POSTGRES_DB=redisgate_dev

# Application
APP_HOST=0.0.0.0
APP_PORT=8080
APP_LOG_LEVEL=debug

# Kubernetes
K8S_NAMESPACE=redisgate-dev
K8S_DOMAIN=redisgate.local

# Minikube
MINIKUBE_DRIVER=docker
MINIKUBE_MEMORY=4096
MINIKUBE_CPUS=2
```

### Service URLs

After starting services, you can access:

- **PostgreSQL**: `localhost:5432`
- **Minikube Dashboard**: `minikube dashboard` (opens in browser)

## Development Workflow

### Typical Development Session

1. **Start external services**:
   ```bash
   ./scripts/dev-services.sh start
   ```

2. **Start Minikube** (if working with Kubernetes):
   ```bash
   ./scripts/minikube-dev.sh start
   ./scripts/minikube-dev.sh namespace
   ```

3. **Develop your application**:
   ```bash
   cargo run  # Run the application
   cargo test # Run tests
   cargo check # Check for errors
   ```

4. **Monitor services**:
   ```bash
   ./scripts/dev-services.sh status
   ./scripts/dev-services.sh logs
   ```

### Database Operations

**Connect to PostgreSQL:**
```bash
./scripts/dev-services.sh psql
```

**Reset database:**
```bash
./scripts/dev-services.sh reset-db
```

### Kubernetes Development

**Build and load images:**
```bash
# Build your application image
docker build -t redisgate:latest .

# Load to Minikube
./scripts/minikube-dev.sh load-images
```

**Deploy to Minikube:**
```bash
kubectl apply -f k8s/
```

## Troubleshooting

### Common Issues

1. **Docker permissions** (Linux):
   ```bash
   sudo usermod -aG docker $USER
   # Log out and back in
   ```

2. **Minikube won't start**:
   ```bash
   ./scripts/minikube-dev.sh delete
   ./scripts/minikube-dev.sh start
   ```

3. **Services won't connect**:
   ```bash
   ./scripts/dev-services.sh clean
   ```

4. **Port conflicts**:
   Edit `env.development` to change ports

### Reset Everything

To completely reset the development environment:

```bash
# Stop and remove all services
./scripts/dev-services.sh stop
docker-compose down -v

# Delete Minikube cluster
./scripts/minikube-dev.sh delete

# Restart setup
./setup-dev.sh
```

## System Requirements

- **Operating System**: Linux, macOS, or Windows with WSL2
- **Memory**: 8GB RAM minimum (16GB recommended)
- **Disk Space**: 10GB free space
- **Network**: Internet connection for downloading dependencies

## Dependencies Installed

The setup script will install these tools if not present:

- **Docker** - Container runtime
- **Docker Compose** - Multi-container orchestration
- **Rust** - Programming language (via rustup)
- **kubectl** - Kubernetes CLI
- **Minikube** - Local Kubernetes cluster
- **Cargo tools** - cargo-watch, cargo-edit

## Next Steps

After setup, you can:

1. **Initialize Rust project structure** (if not done):
   ```bash
   cargo init --name redisgate
   ```

2. **Start coding** your RedisGate application

3. **Create Kubernetes manifests** in `k8s/` directory

4. **Set up CI/CD** pipelines

5. **Add more development tools** as needed

For more information about the RedisGate project, see the main [README.md](../README.md).