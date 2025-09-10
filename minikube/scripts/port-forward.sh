#!/usr/bin/env bash
set -euo pipefail

# Port forward Redis service to localhost
# This script creates a port forward from localhost to Redis service

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="${SCRIPT_DIR}/../config"

# Source configuration
source "${CONFIG_DIR}/minikube-config.yaml"

echo "🔗 Setting up port forward for Redis..."

# Check if kubectl context is correct
current_context=$(kubectl config current-context)
if [[ "${current_context}" != "${CLUSTER_NAME}" ]]; then
    echo "❌ Wrong kubectl context. Expected '${CLUSTER_NAME}', got '${current_context}'"
    echo "Run './minikube-setup.sh' first or switch context manually"
    exit 1
fi

# Check if Redis service exists
if ! kubectl get service redis-server -n "${REDIS_NAMESPACE}" >/dev/null 2>&1; then
    echo "❌ Redis service not found in namespace '${REDIS_NAMESPACE}'"
    echo "Run './redis-deploy.sh' first to deploy Redis"
    exit 1
fi

# Kill existing port forward if running
echo "🧹 Cleaning up existing port forwards..."
pkill -f "kubectl.*port-forward.*redis-server" || true
sleep 2

# Start port forward
echo "🚪 Starting port forward..."
echo "  Local: localhost:${LOCAL_PORT}"
echo "  Remote: redis-server:${REDIS_PORT}"
echo ""
echo "🎯 Redis is now accessible at localhost:${LOCAL_PORT}"
echo "💡 Test connection: redis-cli -h localhost -p ${LOCAL_PORT}"
echo "🛑 Press Ctrl+C to stop port forwarding"
echo ""

# Start port forward (this will run in foreground)
kubectl port-forward service/redis-server \
    "${LOCAL_PORT}:${REDIS_PORT}" \
    --namespace="${REDIS_NAMESPACE}"
