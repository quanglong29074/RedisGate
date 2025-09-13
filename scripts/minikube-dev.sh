#!/bin/bash

# RedisGate Minikube Management Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

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

# Load environment variables
load_env() {
    local env_file="$PROJECT_ROOT/env.development"
    if [[ -f "$env_file" ]]; then
        set -a
        source "$env_file"
        set +a
    fi
}

# Start Minikube
start_minikube() {
    log_info "Starting Minikube..."
    
    minikube start \
        --driver="${MINIKUBE_DRIVER:-docker}" \
        --memory="${MINIKUBE_MEMORY:-4096}" \
        --cpus="${MINIKUBE_CPUS:-2}" \
        --kubernetes-version="${MINIKUBE_KUBERNETES_VERSION:-v1.28.0}"
    
    log_success "Minikube started successfully"
    
    # Enable addons
    log_info "Enabling Minikube addons..."
    minikube addons enable ingress
    minikube addons enable dashboard
    minikube addons enable metrics-server
    
    log_success "Addons enabled"
    show_info
}

# Stop Minikube
stop_minikube() {
    log_info "Stopping Minikube..."
    minikube stop
    log_success "Minikube stopped"
}

# Delete Minikube cluster
delete_minikube() {
    log_warning "This will delete the entire Minikube cluster!"
    read -p "Are you sure you want to continue? (y/N): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        log_info "Deleting Minikube cluster..."
        minikube delete
        log_success "Minikube cluster deleted"
    else
        log_info "Operation cancelled"
    fi
}

# Show Minikube status and info
show_info() {
    log_info "Minikube Status:"
    minikube status
    
    echo ""
    log_info "Minikube IP:"
    minikube ip
    
    echo ""
    log_info "Kubernetes Context:"
    kubectl config current-context
    
    echo ""
    log_info "Useful Commands:"
    echo "  minikube dashboard  - Open Kubernetes dashboard"
    echo "  kubectl get nodes   - List cluster nodes"
    echo "  kubectl get pods -A - List all pods"
    echo ""
    echo "Access URLs:"
    echo "  Dashboard: minikube dashboard"
    echo "  Service URL: minikube service <service-name> --url"
}

# Open Minikube dashboard
open_dashboard() {
    log_info "Opening Minikube dashboard..."
    minikube dashboard
}

# Show cluster info
cluster_info() {
    log_info "Cluster Information:"
    kubectl cluster-info
    
    echo ""
    log_info "Node Information:"
    kubectl get nodes -o wide
    
    echo ""
    log_info "Namespace Information:"
    kubectl get namespaces
}

# Create development namespace
create_namespace() {
    local namespace="${K8S_NAMESPACE:-redisgate-dev}"
    
    log_info "Creating namespace: $namespace"
    
    kubectl create namespace "$namespace" --dry-run=client -o yaml | kubectl apply -f -
    kubectl config set-context --current --namespace="$namespace"
    
    log_success "Namespace '$namespace' ready and set as default"
}

# Enable local registry
enable_registry() {
    log_info "Enabling local Docker registry..."
    
    # Check if registry addon is available
    if minikube addons list | grep -q registry; then
        minikube addons enable registry
        log_success "Registry addon enabled"
    else
        log_warning "Registry addon not available, setting up manual registry..."
        
        # Run local registry if not running
        if ! docker ps | grep -q registry:2; then
            docker run -d -p 5000:5000 --name registry registry:2
            log_success "Local registry started on port 5000"
        else
            log_success "Local registry already running"
        fi
    fi
    
    # Configure Docker environment to use Minikube's Docker daemon
    log_info "Configuring Docker environment..."
    eval "$(minikube docker-env)"
    log_success "Docker environment configured for Minikube"
}

# Load local images to Minikube
load_images() {
    log_info "Loading local Docker images to Minikube..."
    
    # Switch to Minikube's Docker daemon
    eval "$(minikube docker-env)"
    
    # Build RedisGate image if Dockerfile exists
    if [[ -f "$PROJECT_ROOT/Dockerfile" ]]; then
        log_info "Building RedisGate image..."
        docker build -t redisgate:latest "$PROJECT_ROOT"
        log_success "RedisGate image built"
    else
        log_warning "No Dockerfile found, skipping image build"
    fi
}

# Show help
show_help() {
    echo "RedisGate Minikube Management Script"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  start           Start Minikube cluster"
    echo "  stop            Stop Minikube cluster"
    echo "  delete          Delete Minikube cluster"
    echo "  status          Show cluster status and info"
    echo "  dashboard       Open Kubernetes dashboard"
    echo "  info            Show detailed cluster information"
    echo "  namespace       Create development namespace"
    echo "  registry        Enable local Docker registry"
    echo "  load-images     Load local images to Minikube"
    echo "  help            Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 start        Start Minikube with development settings"
    echo "  $0 dashboard    Open Kubernetes dashboard in browser"
    echo "  $0 namespace    Create and switch to dev namespace"
    echo ""
}

# Main function
main() {
    load_env
    
    case "${1:-help}" in
        start)
            start_minikube
            ;;
        stop)
            stop_minikube
            ;;
        delete)
            delete_minikube
            ;;
        status)
            show_info
            ;;
        dashboard)
            open_dashboard
            ;;
        info)
            cluster_info
            ;;
        namespace)
            create_namespace
            ;;
        registry)
            enable_registry
            ;;
        load-images)
            load_images
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            log_error "Unknown command: $1"
            show_help
            exit 1
            ;;
    esac
}

main "$@"