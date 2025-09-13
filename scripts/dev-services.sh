#!/bin/bash

# RedisGate Development Services Management Script

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

# Docker Compose command detection
get_compose_cmd() {
    if docker compose version >/dev/null 2>&1; then
        echo "docker compose"
    elif command -v docker-compose >/dev/null 2>&1; then
        echo "docker-compose"
    else
        log_error "Docker Compose not found!"
        exit 1
    fi
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

# Start all services
start_services() {
    local compose_cmd=$(get_compose_cmd)
    log_info "Starting all development services..."
    cd "$PROJECT_ROOT"
    $compose_cmd up -d
    log_success "Services started successfully"
    show_status
}

# Stop all services
stop_services() {
    local compose_cmd=$(get_compose_cmd)
    log_info "Stopping all development services..."
    cd "$PROJECT_ROOT"
    $compose_cmd down
    log_success "Services stopped successfully"
}

# Restart all services
restart_services() {
    local compose_cmd=$(get_compose_cmd)
    log_info "Restarting all development services..."
    cd "$PROJECT_ROOT"
    $compose_cmd restart
    log_success "Services restarted successfully"
    show_status
}

# Show services status
show_status() {
    local compose_cmd=$(get_compose_cmd)
    log_info "Services status:"
    cd "$PROJECT_ROOT"
    $compose_cmd ps
    
    echo ""
    log_info "Service URLs:"
    echo "  PostgreSQL: ${POSTGRES_HOST:-localhost}:${POSTGRES_PORT:-5432}"
    echo "  Redis: ${REDIS_HOST:-localhost}:${REDIS_PORT:-6379}"
    echo "  Redis Insight: http://localhost:8001"
}

# Show logs
show_logs() {
    local service="$1"
    local compose_cmd=$(get_compose_cmd)
    cd "$PROJECT_ROOT"
    
    if [[ -n "$service" ]]; then
        log_info "Showing logs for service: $service"
        $compose_cmd logs -f "$service"
    else
        log_info "Showing logs for all services"
        $compose_cmd logs -f
    fi
}

# Clean up volumes and restart
clean_restart() {
    local compose_cmd=$(get_compose_cmd)
    log_warning "This will remove all data in PostgreSQL and Redis!"
    read -p "Are you sure you want to continue? (y/N): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        log_info "Stopping services and removing volumes..."
        cd "$PROJECT_ROOT"
        $compose_cmd down -v
        $compose_cmd up -d
        log_success "Clean restart completed"
        show_status
    else
        log_info "Operation cancelled"
    fi
}

# Reset database
reset_database() {
    local compose_cmd=$(get_compose_cmd)
    log_warning "This will reset the PostgreSQL database!"
    read -p "Are you sure you want to continue? (y/N): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        log_info "Resetting PostgreSQL database..."
        cd "$PROJECT_ROOT"
        $compose_cmd exec postgres psql -U "${POSTGRES_USER:-redisgate_dev}" -d "${POSTGRES_DB:-redisgate_dev}" -c "DROP SCHEMA public CASCADE; CREATE SCHEMA public; GRANT ALL ON SCHEMA public TO ${POSTGRES_USER:-redisgate_dev};"
        $compose_cmd restart postgres
        log_success "Database reset completed"
    else
        log_info "Operation cancelled"
    fi
}

# Connect to PostgreSQL
connect_postgres() {
    local compose_cmd=$(get_compose_cmd)
    log_info "Connecting to PostgreSQL..."
    cd "$PROJECT_ROOT"
    $compose_cmd exec postgres psql -U "${POSTGRES_USER:-redisgate_dev}" -d "${POSTGRES_DB:-redisgate_dev}"
}

# Connect to Redis
connect_redis() {
    local compose_cmd=$(get_compose_cmd)
    log_info "Connecting to Redis..."
    cd "$PROJECT_ROOT"
    $compose_cmd exec redis redis-cli -a "${REDIS_PASSWORD:-redisgate_redis_password}"
}

# Show help
show_help() {
    echo "RedisGate Development Services Management"
    echo ""
    echo "Usage: $0 [COMMAND] [OPTIONS]"
    echo ""
    echo "Commands:"
    echo "  start           Start all services"
    echo "  stop            Stop all services"
    echo "  restart         Restart all services"
    echo "  status          Show services status"
    echo "  logs [SERVICE]  Show logs (optional service name)"
    echo "  clean           Clean restart (removes all data)"
    echo "  reset-db        Reset PostgreSQL database"
    echo "  psql            Connect to PostgreSQL"
    echo "  redis-cli       Connect to Redis"
    echo "  help            Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 start        Start all services"
    echo "  $0 logs         Show all logs"
    echo "  $0 logs redis   Show Redis logs only"
    echo "  $0 psql         Connect to PostgreSQL"
    echo ""
}

# Main function
main() {
    load_env
    
    case "${1:-help}" in
        start)
            start_services
            ;;
        stop)
            stop_services
            ;;
        restart)
            restart_services
            ;;
        status)
            show_status
            ;;
        logs)
            show_logs "$2"
            ;;
        clean)
            clean_restart
            ;;
        reset-db)
            reset_database
            ;;
        psql)
            connect_postgres
            ;;
        redis-cli)
            connect_redis
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