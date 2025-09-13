# RedisGate Makefile
# Provides common development commands for the RedisGate project

.PHONY: help setup clean dev build test lint check deploy logs

# Load environment variables
include env.development
export

# Colors for output
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[1;33m
RED := \033[0;31m
NC := \033[0m

# Default target
help: ## Show this help message
	@echo "$(BLUE)RedisGate Development Commands$(NC)"
	@echo "================================"
	@echo ""
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "$(GREEN)%-15s$(NC) %s\n", $$1, $$2}' $(MAKEFILE_LIST)
	@echo ""
	@echo "$(YELLOW)Examples:$(NC)"
	@echo "  make setup      # Set up development environment"
	@echo "  make dev        # Start development services"
	@echo "  make build      # Build the application"
	@echo "  make deploy     # Deploy to Minikube"

setup: ## Set up the complete development environment
	@echo "$(BLUE)Setting up development environment...$(NC)"
	./setup-dev.sh

setup-docker: ## Set up only Docker and external services
	@echo "$(BLUE)Setting up Docker and external services...$(NC)"
	./setup-dev.sh --docker-only

setup-rust: ## Set up only Rust toolchain
	@echo "$(BLUE)Setting up Rust toolchain...$(NC)"
	./setup-dev.sh --rust-only

setup-k8s: ## Set up only Kubernetes tools
	@echo "$(BLUE)Setting up Kubernetes tools...$(NC)"
	./setup-dev.sh --k8s-only

clean: ## Clean up all development resources
	@echo "$(YELLOW)Cleaning up development resources...$(NC)"
	./scripts/dev-services.sh stop || true
	./scripts/minikube-dev.sh delete || true
	docker system prune -f || true
	@echo "$(GREEN)Cleanup completed$(NC)"

# Development services
dev: ## Start all development services
	@echo "$(BLUE)Starting development services...$(NC)"
	./scripts/dev-services.sh start

dev-stop: ## Stop all development services
	@echo "$(BLUE)Stopping development services...$(NC)"
	./scripts/dev-services.sh stop

dev-restart: ## Restart all development services
	@echo "$(BLUE)Restarting development services...$(NC)"
	./scripts/dev-services.sh restart

dev-logs: ## Show development services logs
	@echo "$(BLUE)Showing development services logs...$(NC)"
	./scripts/dev-services.sh logs

dev-status: ## Show development services status
	@echo "$(BLUE)Development services status:$(NC)"
	./scripts/dev-services.sh status

# Database operations
db-connect: ## Connect to PostgreSQL database
	@echo "$(BLUE)Connecting to PostgreSQL...$(NC)"
	./scripts/dev-services.sh psql

db-reset: ## Reset PostgreSQL database
	@echo "$(YELLOW)Resetting PostgreSQL database...$(NC)"
	./scripts/dev-services.sh reset-db

redis-connect: ## Connect to Redis
	@echo "$(BLUE)Connecting to Redis...$(NC)"
	./scripts/dev-services.sh redis-cli

# Minikube operations
k8s-start: ## Start Minikube cluster
	@echo "$(BLUE)Starting Minikube cluster...$(NC)"
	./scripts/minikube-dev.sh start

k8s-stop: ## Stop Minikube cluster
	@echo "$(BLUE)Stopping Minikube cluster...$(NC)"
	./scripts/minikube-dev.sh stop

k8s-status: ## Show Minikube cluster status
	@echo "$(BLUE)Minikube cluster status:$(NC)"
	./scripts/minikube-dev.sh status

k8s-dashboard: ## Open Kubernetes dashboard
	@echo "$(BLUE)Opening Kubernetes dashboard...$(NC)"
	./scripts/minikube-dev.sh dashboard

k8s-namespace: ## Create development namespace
	@echo "$(BLUE)Creating development namespace...$(NC)"
	./scripts/minikube-dev.sh namespace

# Rust operations
build: ## Build the Rust application
	@echo "$(BLUE)Building RedisGate application...$(NC)"
	cargo build

build-release: ## Build the Rust application in release mode
	@echo "$(BLUE)Building RedisGate application (release)...$(NC)"
	cargo build --release

test: ## Run tests
	@echo "$(BLUE)Running tests...$(NC)"
	cargo test

test-watch: ## Run tests in watch mode
	@echo "$(BLUE)Running tests in watch mode...$(NC)"
	cargo watch -x test

lint: ## Run Rust linter (clippy)
	@echo "$(BLUE)Running Rust linter...$(NC)"
	cargo clippy -- -D warnings

format: ## Format Rust code
	@echo "$(BLUE)Formatting Rust code...$(NC)"
	cargo fmt

check: ## Check Rust code without building
	@echo "$(BLUE)Checking Rust code...$(NC)"
	cargo check

run: ## Run the application in development mode
	@echo "$(BLUE)Running RedisGate application...$(NC)"
	cargo run

run-watch: ## Run the application with auto-reload
	@echo "$(BLUE)Running RedisGate application with auto-reload...$(NC)"
	cargo watch -x run

# Docker operations
docker-build: ## Build Docker image
	@echo "$(BLUE)Building Docker image...$(NC)"
	docker build -t redisgate:latest .

docker-build-dev: ## Build Docker image for development
	@echo "$(BLUE)Building Docker image for development...$(NC)"
	docker build -t redisgate:dev -f Dockerfile.dev .

docker-push: ## Push Docker image to registry
	@echo "$(BLUE)Pushing Docker image to registry...$(NC)"
	docker tag redisgate:latest $(DOCKER_REGISTRY)/redisgate:$(DOCKER_TAG)
	docker push $(DOCKER_REGISTRY)/redisgate:$(DOCKER_TAG)

# Kubernetes deployment
deploy: k8s-start ## Deploy application to Minikube
	@echo "$(BLUE)Deploying RedisGate to Minikube...$(NC)"
	eval $$(minikube docker-env) && docker build -t redisgate:latest .
	kubectl apply -f k8s/development.yaml
	@echo "$(GREEN)Deployment completed$(NC)"
	@echo "$(YELLOW)Add '127.0.0.1 redisgate.local' to your /etc/hosts file$(NC)"
	@echo "$(YELLOW)Then access the application at: http://redisgate.local$(NC)"

deploy-clean: ## Clean deploy (delete and redeploy)
	@echo "$(BLUE)Clean deploying RedisGate to Minikube...$(NC)"
	kubectl delete -f k8s/development.yaml --ignore-not-found
	sleep 5
	$(MAKE) deploy

undeploy: ## Remove deployment from Minikube
	@echo "$(BLUE)Removing RedisGate deployment from Minikube...$(NC)"
	kubectl delete -f k8s/development.yaml --ignore-not-found
	@echo "$(GREEN)Undeployment completed$(NC)"

# Logging and monitoring
logs: ## Show application logs from Kubernetes
	@echo "$(BLUE)Showing RedisGate application logs...$(NC)"
	kubectl logs -f deployment/redisgate -n $(K8S_NAMESPACE)

logs-tail: ## Tail application logs from Kubernetes
	@echo "$(BLUE)Tailing RedisGate application logs...$(NC)"
	kubectl logs -f deployment/redisgate -n $(K8S_NAMESPACE) --tail=100

describe: ## Describe Kubernetes resources
	@echo "$(BLUE)Describing Kubernetes resources...$(NC)"
	kubectl describe deployment redisgate -n $(K8S_NAMESPACE)
	kubectl describe service redisgate-service -n $(K8S_NAMESPACE)
	kubectl describe ingress redisgate-ingress -n $(K8S_NAMESPACE)

# Port forwarding
port-forward: ## Forward local port to application
	@echo "$(BLUE)Forwarding port 8080 to RedisGate service...$(NC)"
	kubectl port-forward service/redisgate-service 8080:80 -n $(K8S_NAMESPACE)

# Complete development workflow
dev-full: setup dev k8s-start deploy ## Complete development setup and deployment
	@echo "$(GREEN)Full development environment is ready!$(NC)"
	@echo ""
	@echo "$(YELLOW)Services:$(NC)"
	@echo "  PostgreSQL: localhost:$(POSTGRES_PORT)"
	@echo "  Redis: localhost:$(REDIS_PORT)"
	@echo "  Redis Insight: http://localhost:8001"
	@echo ""
	@echo "$(YELLOW)Kubernetes:$(NC)"
	@echo "  Dashboard: minikube dashboard"
	@echo "  Application: http://redisgate.local (add to /etc/hosts)"
	@echo ""
	@echo "$(YELLOW)Development:$(NC)"
	@echo "  make run        # Run application locally"
	@echo "  make test       # Run tests"
	@echo "  make logs       # View K8s logs"

# CI/CD simulation
ci: lint test build ## Simulate CI pipeline (lint, test, build)
	@echo "$(GREEN)CI pipeline completed successfully$(NC)"

# Health checks
health: ## Check health of all services
	@echo "$(BLUE)Checking service health...$(NC)"
	@echo "$(YELLOW)External Services:$(NC)"
	./scripts/dev-services.sh status
	@echo ""
	@echo "$(YELLOW)Kubernetes:$(NC)"
	./scripts/minikube-dev.sh status || echo "$(RED)Minikube not running$(NC)"
	@echo ""
	@echo "$(YELLOW)Application:$(NC)"
	kubectl get pods -n $(K8S_NAMESPACE) 2>/dev/null || echo "$(RED)No pods found$(NC)"