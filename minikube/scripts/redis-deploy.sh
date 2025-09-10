#!/usr/bin/env bash
set -euo pipefail

# Deploy Redis server to minikube cluster
# This script creates ConfigMap, Deployment, and Service for Redis

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="${SCRIPT_DIR}/../config"
TEMPLATES_DIR="${SCRIPT_DIR}/../templates"

# Source configuration
source "${CONFIG_DIR}/minikube-config.yaml"

echo "🚀 Deploying Redis to minikube cluster..."

# Check if kubectl context is correct
current_context=$(kubectl config current-context)
if [[ "${current_context}" != "${CLUSTER_NAME}" ]]; then
    echo "❌ Wrong kubectl context. Expected '${CLUSTER_NAME}', got '${current_context}'"
    echo "Run './minikube-setup.sh' first or switch context manually"
    exit 1
fi

# Create ConfigMap from redis config
echo "📄 Creating Redis ConfigMap..."
kubectl create configmap redis-config \
    --from-file=redis.conf="${CONFIG_DIR}/redis-config.yaml" \
    --namespace="${REDIS_NAMESPACE}" \
    --dry-run=client -o yaml | kubectl apply -f -

# Apply Redis deployment and service
echo "🐳 Deploying Redis..."
kubectl apply -f "${TEMPLATES_DIR}/redis-deployment.yaml" -n "${REDIS_NAMESPACE}"
kubectl apply -f "${TEMPLATES_DIR}/redis-service.yaml" -n "${REDIS_NAMESPACE}"

# Wait for deployment to be ready
echo "⏳ Waiting for Redis deployment to be ready..."
kubectl wait --for=condition=Available deployment/redis-server \
    --namespace="${REDIS_NAMESPACE}" \
    --timeout=300s

# Get pod status
echo "📊 Redis deployment status:"
kubectl get pods -l app=redis-server -n "${REDIS_NAMESPACE}"

echo ""
echo "✅ Redis deployment completed!"
echo ""
echo "🔗 Connection information:"
echo "  Service: redis-server.${REDIS_NAMESPACE}.svc.cluster.local"
echo "  Port: ${REDIS_PORT}"
echo ""
echo "🎯 Next steps:"
echo "  1. Run './port-forward.sh' to access Redis from localhost"
echo "  2. Test connection: redis-cli -h localhost -p ${LOCAL_PORT}"
