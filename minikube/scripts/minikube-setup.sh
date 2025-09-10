#!/usr/bin/env bash
set -euo pipefail

# Setup Minikube cluster for Redis development
# This script creates a minikube cluster with necessary addons

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
CONFIG_DIR="${SCRIPT_DIR}/../config"

# Source configuration
source "${CONFIG_DIR}/minikube-config.yaml"

echo "🚀 Setting up Minikube cluster for Redis development..."

# Function to check if command exists
require_cmd() {
    command -v "$1" >/dev/null 2>&1 || { 
        echo "❌ Error: '$1' is required but not installed" >&2
        exit 1
    }
}

# Check prerequisites
echo "📋 Checking prerequisites..."
require_cmd minikube
require_cmd kubectl
require_cmd docker

# Start minikube cluster
echo "🔧 Starting minikube cluster '${CLUSTER_NAME}'..."
if minikube status -p "${CLUSTER_NAME}" >/dev/null 2>&1; then
    echo "✅ Minikube cluster '${CLUSTER_NAME}' is already running"
else
    echo "🆕 Creating new minikube cluster..."
    minikube start \
        --profile="${CLUSTER_NAME}" \
        --driver="${DRIVER}" \
        --cpus="${CPUS}" \
        --memory="${MEMORY}" \
        --disk-size="${DISK_SIZE}" \
        --kubernetes-version="${KUBERNETES_VERSION}"
fi

# Set kubectl context
echo "🔧 Setting kubectl context..."
kubectl config use-context "${CLUSTER_NAME}"

# Enable addons
echo "🔌 Enabling addons..."
for addon in "${ADDONS[@]}"; do
    echo "  Enabling ${addon}..."
    minikube addons enable "${addon}" -p "${CLUSTER_NAME}"
done

# Create namespace
echo "📦 Creating namespace '${REDIS_NAMESPACE}'..."
kubectl create namespace "${REDIS_NAMESPACE}" --dry-run=client -o yaml | kubectl apply -f -

# Wait for cluster to be ready
echo "⏳ Waiting for cluster to be ready..."
kubectl wait --for=condition=Ready nodes --all --timeout=300s

echo "✅ Minikube cluster setup completed!"
echo ""
echo "📊 Cluster Information:"
echo "  Cluster Name: ${CLUSTER_NAME}"
echo "  Driver: ${DRIVER}"
echo "  CPUs: ${CPUS}"
echo "  Memory: ${MEMORY}"
echo "  Namespace: ${REDIS_NAMESPACE}"
echo ""
echo "🎯 Next steps:"
echo "  1. Run './redis-deploy.sh' to deploy Redis"
echo "  2. Run './port-forward.sh' to access Redis"
echo "  3. Run 'minikube dashboard -p ${CLUSTER_NAME}' to open dashboard"
