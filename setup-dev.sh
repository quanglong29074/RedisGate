#!/bin/bash

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"

# Load environment variables
ENV_FILE="$PROJECT_ROOT/env.development"

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Load environment variables from env.development
load_env() {
    if [[ -f "$ENV_FILE" ]]; then
        log_info "Loading environment variables from $ENV_FILE"
        set -a
        source "$ENV_FILE"
        set +a
        log_success "Environment variables loaded"
    else
        log_error "Environment file $ENV_FILE not found!"
        exit 1
    fi
}

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Check and install Docker
setup_docker() {
    log_info "Checking Docker installation..."
    
    if command_exists docker; then
        log_success "Docker is already installed"
        docker --version
    else
        log_warning "Docker not found. Installing Docker..."
        
        # Detect OS
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            # Linux installation
            curl -fsSL https://get.docker.com -o get-docker.sh
            sudo sh get-docker.sh
            sudo usermod -aG docker "$USER"
            rm get-docker.sh
            
            # Install Docker Compose
            sudo curl -L "https://github.com/docker/compose/releases/latest/download/docker-compose-$(uname -s)-$(uname -m)" -o /usr/local/bin/docker-compose
            sudo chmod +x /usr/local/bin/docker-compose
            
            log_success "Docker and Docker Compose installed successfully"
            log_warning "Please log out and back in for Docker group changes to take effect"
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            # macOS installation
            if command_exists brew; then
                brew install --cask docker
                log_success "Docker installed via Homebrew"
            else
                log_error "Please install Docker Desktop for macOS manually from https://docker.com"
                exit 1
            fi
        else
            log_error "Unsupported operating system for automatic Docker installation"
            log_info "Please install Docker manually from https://docker.com"
            exit 1
        fi
    fi
    
    # Check Docker Compose
    if docker compose version >/dev/null 2>&1; then
        log_success "Docker Compose is available"
    elif command_exists docker-compose; then
        log_success "Docker Compose (legacy) is available"
    else
        log_error "Docker Compose not found. Please install Docker Compose"
        exit 1
    fi
}

# Check and install Rust
setup_rust() {
    log_info "Checking Rust installation..."
    
    if command_exists rustc && command_exists cargo; then
        log_success "Rust is already installed"
        rustc --version
        cargo --version
    else
        log_warning "Rust not found. Installing Rust..."
        curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
        source "$HOME/.cargo/env"
        log_success "Rust installed successfully"
    fi
    
    # Install additional Rust components
    log_info "Installing additional Rust components..."
    rustup component add clippy rustfmt
    
    # Install useful cargo tools
    cargo install cargo-watch cargo-edit
    
    log_success "Rust setup completed"
}

# Check and install kubectl
setup_kubectl() {
    log_info "Checking kubectl installation..."
    
    if command_exists kubectl; then
        log_success "kubectl is already installed"
        kubectl version --client
    else
        log_warning "kubectl not found. Installing kubectl..."
        
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/linux/amd64/kubectl"
            sudo install -o root -g root -m 0755 kubectl /usr/local/bin/kubectl
            rm kubectl
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            if command_exists brew; then
                brew install kubectl
            else
                curl -LO "https://dl.k8s.io/release/$(curl -L -s https://dl.k8s.io/release/stable.txt)/bin/darwin/amd64/kubectl"
                chmod +x kubectl
                sudo mv kubectl /usr/local/bin/
            fi
        fi
        
        log_success "kubectl installed successfully"
    fi
}

# Check and install Minikube
setup_minikube() {
    log_info "Checking Minikube installation..."
    
    if command_exists minikube; then
        log_success "Minikube is already installed"
        minikube version
    else
        log_warning "Minikube not found. Installing Minikube..."
        
        if [[ "$OSTYPE" == "linux-gnu"* ]]; then
            curl -LO https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64
            sudo install minikube-linux-amd64 /usr/local/bin/minikube
            rm minikube-linux-amd64
        elif [[ "$OSTYPE" == "darwin"* ]]; then
            if command_exists brew; then
                brew install minikube
            else
                curl -LO https://storage.googleapis.com/minikube/releases/latest/minikube-darwin-amd64
                sudo install minikube-darwin-amd64 /usr/local/bin/minikube
                rm minikube-darwin-amd64
            fi
        fi
        
        log_success "Minikube installed successfully"
    fi
    
    # Start Minikube if not running
    log_info "Checking Minikube status..."
    if minikube status >/dev/null 2>&1; then
        log_success "Minikube is already running"
    else
        log_info "Starting Minikube..."
        minikube start \
            --driver="${MINIKUBE_DRIVER:-docker}" \
            --memory="${MINIKUBE_MEMORY:-4096}" \
            --cpus="${MINIKUBE_CPUS:-2}" \
            --kubernetes-version="${MINIKUBE_KUBERNETES_VERSION:-v1.28.0}"
        
        log_success "Minikube started successfully"
    fi
    
    # Enable addons
    log_info "Enabling Minikube addons..."
    minikube addons enable ingress
    minikube addons enable dashboard
    minikube addons enable metrics-server
    
    log_success "Minikube setup completed"
}

# Setup external dependencies with Docker Compose
setup_external_dependencies() {
    log_info "Setting up external dependencies (PostgreSQL, Redis)..."
    
    # Detect Docker Compose command
    local compose_cmd
    if docker compose version >/dev/null 2>&1; then
        compose_cmd="docker compose"
    elif command_exists docker-compose; then
        compose_cmd="docker-compose"
    else
        log_error "Docker Compose not found!"
        exit 1
    fi
    
    cd "$PROJECT_ROOT"
    
    # Check if services are already running
    if $compose_cmd ps | grep -q "Up"; then
        log_info "Some services are already running. Checking health..."
        $compose_cmd ps
    fi
    
    # Start services
    log_info "Starting PostgreSQL and Redis services..."
    $compose_cmd up -d
    
    # Wait for services to be healthy
    log_info "Waiting for services to be ready..."
    sleep 10
    
    # Check PostgreSQL connection
    local max_attempts=30
    local attempt=1
    
    while [[ $attempt -le $max_attempts ]]; do
        if $compose_cmd exec -T postgres pg_isready -U "${POSTGRES_USER}" >/dev/null 2>&1; then
            log_success "PostgreSQL is ready"
            break
        fi
        
        if [[ $attempt -eq $max_attempts ]]; then
            log_error "PostgreSQL failed to start after $max_attempts attempts"
            exit 1
        fi
        
        log_info "Waiting for PostgreSQL... (attempt $attempt/$max_attempts)"
        sleep 2
        ((attempt++))
    done
    
    # Check Redis connection
    attempt=1
    while [[ $attempt -le $max_attempts ]]; do
        if $compose_cmd exec -T redis redis-cli -a "${REDIS_PASSWORD}" ping >/dev/null 2>&1; then
            log_success "Redis is ready"
            break
        fi
        
        if [[ $attempt -eq $max_attempts ]]; then
            log_error "Redis failed to start after $max_attempts attempts"
            exit 1
        fi
        
        log_info "Waiting for Redis... (attempt $attempt/$max_attempts)"
        sleep 2
        ((attempt++))
    done
    
    log_success "External dependencies are ready"
    
    # Show connection information
    echo ""
    log_info "=== Connection Information ==="
    echo "PostgreSQL:"
    echo "  Host: ${POSTGRES_HOST}:${POSTGRES_PORT}"
    echo "  Database: ${POSTGRES_DB}"
    echo "  Username: ${POSTGRES_USER}"
    echo "  Password: ${POSTGRES_PASSWORD}"
    echo ""
    echo "Redis:"
    echo "  Host: ${REDIS_HOST}:${REDIS_PORT}"
    echo "  Password: ${REDIS_PASSWORD}"
    echo ""
    echo "Redis Insight (Web UI): http://localhost:8001"
    echo ""
}

# Create basic Rust project structure if it doesn't exist
setup_rust_project() {
    log_info "Setting up Rust project structure..."
    
    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        log_info "Creating basic Rust project structure..."
        
        cd "$PROJECT_ROOT"
        cargo init --name redisgate
        
        # Update Cargo.toml with common dependencies
        cat > Cargo.toml << 'EOF'
[package]
name = "redisgate"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
clap = { version = "4.0", features = ["derive"] }
tracing = "0.1"
tracing-subscriber = "0.3"
uuid = { version = "1.0", features = ["v4", "serde"] }
chrono = { version = "0.4", features = ["serde"] }

# Database
sqlx = { version = "0.7", features = ["runtime-tokio-native-tls", "postgres", "uuid", "chrono"] }

# Redis
redis = { version = "0.23", features = ["tokio-comp"] }

# Web framework
axum = "0.7"
tower = "0.4"
tower-http = { version = "0.5", features = ["cors", "trace"] }

# Config
config = "0.13"
dotenv = "0.15"

# Kubernetes client
kube = "0.87"
k8s-openapi = { version = "0.20", features = ["v1_28"] }

[dev-dependencies]
tempfile = "3.0"
EOF
        
        log_success "Cargo.toml created with common dependencies"
    else
        log_success "Cargo.toml already exists"
    fi
}

# Display help information
show_help() {
    echo "RedisGate Development Environment Setup Script"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --help, -h          Show this help message"
    echo "  --docker-only       Setup only Docker and external dependencies"
    echo "  --rust-only         Setup only Rust toolchain"
    echo "  --k8s-only          Setup only Kubernetes tools (kubectl, minikube)"
    echo "  --no-minikube       Skip Minikube setup"
    echo "  --no-services       Skip starting external services"
    echo ""
    echo "Examples:"
    echo "  $0                  Full setup (recommended)"
    echo "  $0 --docker-only    Setup only Docker and start services"
    echo "  $0 --no-services    Setup tools but don't start services"
    echo ""
}

# Main function
main() {
    local docker_only=false
    local rust_only=false
    local k8s_only=false
    local no_minikube=false
    local no_services=false
    
    # Parse command line arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --help|-h)
                show_help
                exit 0
                ;;
            --docker-only)
                docker_only=true
                shift
                ;;
            --rust-only)
                rust_only=true
                shift
                ;;
            --k8s-only)
                k8s_only=true
                shift
                ;;
            --no-minikube)
                no_minikube=true
                shift
                ;;
            --no-services)
                no_services=true
                shift
                ;;
            *)
                log_error "Unknown option: $1"
                show_help
                exit 1
                ;;
        esac
    done
    
    echo "🚀 RedisGate Development Environment Setup"
    echo "=========================================="
    echo ""
    
    # Load environment variables
    load_env
    
    # Execute setup based on options
    if [[ "$docker_only" == true ]]; then
        setup_docker
        if [[ "$no_services" == false ]]; then
            setup_external_dependencies
        fi
    elif [[ "$rust_only" == true ]]; then
        setup_rust
        setup_rust_project
    elif [[ "$k8s_only" == true ]]; then
        setup_kubectl
        if [[ "$no_minikube" == false ]]; then
            setup_minikube
        fi
    else
        # Full setup
        setup_docker
        setup_rust
        setup_kubectl
        
        if [[ "$no_minikube" == false ]]; then
            setup_minikube
        fi
        
        setup_rust_project
        
        if [[ "$no_services" == false ]]; then
            setup_external_dependencies
        fi
    fi
    
    echo ""
    echo "🎉 Setup completed successfully!"
    echo ""
    echo "Next steps:"
    echo "1. Source your shell configuration: source ~/.bashrc (or ~/.zshrc)"
    echo "2. Verify installations:"
    echo "   - Docker: docker --version"
    echo "   - Rust: cargo --version"
    echo "   - kubectl: kubectl version --client"
    if [[ "$no_minikube" == false ]]; then
        echo "   - Minikube: minikube status"
    fi
    echo "3. Start coding your RedisGate application!"
    echo ""
    echo "Configuration files:"
    echo "  - Environment: $ENV_FILE"
    echo "  - Docker Compose: $PROJECT_ROOT/docker-compose.yml"
    echo "  - Database Init: $PROJECT_ROOT/scripts/init-db.sql"
    echo ""
}

# Run main function
main "$@"