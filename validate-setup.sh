#!/bin/bash

# RedisGate Development Environment Validation Script
# This script validates that the development environment is properly set up

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$SCRIPT_DIR"

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

# Check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Test Docker installation
test_docker() {
    log_info "Testing Docker installation..."
    
    if command_exists docker; then
        docker --version
        log_success "Docker is installed and accessible"
    else
        log_error "Docker is not installed or not accessible"
        return 1
    fi
    
    # Test Docker Compose
    if docker compose version >/dev/null 2>&1; then
        docker compose version
        log_success "Docker Compose is available"
    elif command_exists docker-compose; then
        docker-compose --version
        log_success "Docker Compose (legacy) is available"
    else
        log_error "Docker Compose is not available"
        return 1
    fi
}

# Test Rust installation
test_rust() {
    log_info "Testing Rust installation..."
    
    if command_exists rustc && command_exists cargo; then
        rustc --version
        cargo --version
        log_success "Rust toolchain is installed"
    else
        log_error "Rust toolchain is not installed"
        return 1
    fi
    
    # Test Rust components
    if rustup component list --installed | grep -q clippy; then
        log_success "Clippy is installed"
    else
        log_warning "Clippy is not installed"
    fi
    
    if rustup component list --installed | grep -q rustfmt; then
        log_success "rustfmt is installed"
    else
        log_warning "rustfmt is not installed"
    fi
}

# Test Kubernetes tools
test_kubernetes() {
    log_info "Testing Kubernetes tools..."
    
    if command_exists kubectl; then
        kubectl version --client
        log_success "kubectl is installed"
    else
        log_error "kubectl is not installed"
        return 1
    fi
    
    if command_exists minikube; then
        minikube version
        log_success "Minikube is installed"
        
        # Check if Minikube is running
        if minikube status >/dev/null 2>&1; then
            log_success "Minikube cluster is running"
        else
            log_warning "Minikube cluster is not running"
        fi
    else
        log_error "Minikube is not installed"
        return 1
    fi
}

# Test configuration files
test_configuration() {
    log_info "Testing configuration files..."
    
    # Check env.development
    if [[ -f "$PROJECT_ROOT/env.development" ]]; then
        log_success "env.development file exists"
        
        # Load and test key variables
        source "$PROJECT_ROOT/env.development"
        
        if [[ -n "$POSTGRES_HOST" && -n "$REDIS_HOST" ]]; then
            log_success "Environment variables are properly set"
        else
            log_error "Required environment variables are missing"
            return 1
        fi
    else
        log_error "env.development file is missing"
        return 1
    fi
    
    # Check Docker Compose file
    if [[ -f "$PROJECT_ROOT/docker-compose.yml" ]]; then
        log_success "docker-compose.yml file exists"
        
        # Validate Docker Compose configuration
        cd "$PROJECT_ROOT"
        if docker compose config --quiet 2>/dev/null; then
            log_success "Docker Compose configuration is valid"
        else
            log_error "Docker Compose configuration is invalid"
            return 1
        fi
    else
        log_error "docker-compose.yml file is missing"
        return 1
    fi
}

# Test scripts
test_scripts() {
    log_info "Testing setup scripts..."
    
    local scripts=(
        "setup-dev.sh"
        "scripts/dev-services.sh"
        "scripts/minikube-dev.sh"
    )
    
    for script in "${scripts[@]}"; do
        if [[ -f "$PROJECT_ROOT/$script" && -x "$PROJECT_ROOT/$script" ]]; then
            log_success "$script exists and is executable"
        else
            log_error "$script is missing or not executable"
            return 1
        fi
    done
    
    # Test script help functions
    if "$PROJECT_ROOT/setup-dev.sh" --help >/dev/null 2>&1; then
        log_success "setup-dev.sh help function works"
    else
        log_error "setup-dev.sh help function failed"
        return 1
    fi
}

# Test external services (if running)
test_services() {
    log_info "Testing external services..."
    
    # Load environment variables
    source "$PROJECT_ROOT/env.development"
    
    # Test PostgreSQL
    if docker ps | grep -q redisgate-postgres; then
        log_success "PostgreSQL container is running"
        
        # Test connection
        if docker exec redisgate-postgres pg_isready -U "$POSTGRES_USER" >/dev/null 2>&1; then
            log_success "PostgreSQL is accepting connections"
        else
            log_warning "PostgreSQL is not ready for connections"
        fi
    else
        log_warning "PostgreSQL container is not running"
    fi
    
    # Test Redis
    if docker ps | grep -q redisgate-redis; then
        log_success "Redis container is running"
        
        # Test connection
        if docker exec redisgate-redis redis-cli -a "$REDIS_PASSWORD" ping >/dev/null 2>&1; then
            log_success "Redis is accepting connections"
        else
            log_warning "Redis is not ready for connections"
        fi
    else
        log_warning "Redis container is not running"
    fi
}

# Test Makefile
test_makefile() {
    log_info "Testing Makefile..."
    
    if [[ -f "$PROJECT_ROOT/Makefile" ]]; then
        log_success "Makefile exists"
        
        # Test help command
        cd "$PROJECT_ROOT"
        if make help >/dev/null 2>&1; then
            log_success "Makefile help command works"
        else
            log_error "Makefile help command failed"
            return 1
        fi
    else
        log_error "Makefile is missing"
        return 1
    fi
}

# Main validation function
main() {
    echo "🧪 RedisGate Development Environment Validation"
    echo "==============================================="
    echo ""
    
    local errors=0
    
    # Run all tests
    test_docker || ((errors++))
    echo ""
    
    test_rust || ((errors++))
    echo ""
    
    test_kubernetes || ((errors++))
    echo ""
    
    test_configuration || ((errors++))
    echo ""
    
    test_scripts || ((errors++))
    echo ""
    
    test_makefile || ((errors++))
    echo ""
    
    test_services || ((errors++))
    echo ""
    
    # Summary
    echo "============================================="
    if [[ $errors -eq 0 ]]; then
        log_success "All validation tests passed! ✅"
        echo ""
        echo "Your development environment is ready for RedisGate development."
        echo ""
        echo "Next steps:"
        echo "  make dev        # Start development services"
        echo "  make deploy     # Deploy to Minikube"
        echo "  make run        # Run the application"
        echo ""
    else
        log_error "$errors validation test(s) failed! ❌"
        echo ""
        echo "Please fix the issues above and run this script again."
        echo "You can also try running './setup-dev.sh' to reinstall components."
        echo ""
        exit 1
    fi
}

main "$@"