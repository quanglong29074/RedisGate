#!/usr/bin/env bash
set -euo pipefail

# Creates a local Kind cluster for development with:
# - Local Docker registry (localhost:REG_PORT)
# - Ingress-NGINX controller
# - Port mappings 80->8080, 443->8443 (from kind-config.yaml)

KIND_CLUSTER_NAME="${KIND_CLUSTER_NAME:-redis-http-dev}"
REG_NAME="${REG_NAME:-kind-registry}"
REG_PORT="${REG_PORT:-5001}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"
KIND_CONFIG="${KIND_CONFIG:-${ROOT_DIR}/kind-config.yaml}"

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "Error: '$1' is required but not installed" >&2; exit 1; }
}

echo "[check] prerequisites"
require_cmd docker
require_cmd kind
require_cmd kubectl

echo "[registry] ensuring local registry '${REG_NAME}' on 127.0.0.1:${REG_PORT}"
if docker ps -q -f name=^/${REG_NAME}$ >/dev/null; then
  echo "[registry] already running"
elif docker ps -aq -f name=^/${REG_NAME}$ >/dev/null; then
  echo "[registry] starting existing container"
  docker start "${REG_NAME}" >/dev/null
else
  echo "[registry] creating new container"
  docker run -d --restart=always -p "127.0.0.1:${REG_PORT}:5000" --name "${REG_NAME}" registry:2 >/dev/null
fi

echo "[kind] creating cluster '${KIND_CLUSTER_NAME}' if missing"
if kind get clusters | grep -qx "${KIND_CLUSTER_NAME}"; then
  echo "[kind] cluster exists, skipping create"
else
  kind create cluster --name "${KIND_CLUSTER_NAME}" --config "${KIND_CONFIG}"
fi

echo "[network] connecting registry to 'kind' network (ignore if already connected)"
docker network connect "kind" "${REG_NAME}" 2>/dev/null || true

echo "[k8s] publishing local-registry-hosting ConfigMap"
cat <<EOF | kubectl apply -f -
apiVersion: v1
kind: ConfigMap
metadata:
  name: local-registry-hosting
  namespace: kube-public
data:
  localRegistryHosting.v1: |
    host: "localhost:${REG_PORT}"
    help: "https://kind.sigs.k8s.io/docs/user/local-registry/"
EOF

echo "[ingress] installing ingress-nginx for Kind"
kubectl apply -f https://raw.githubusercontent.com/kubernetes/ingress-nginx/main/deploy/static/provider/kind/deploy.yaml

echo "[ingress] waiting for controller to be Ready"
kubectl wait --namespace ingress-nginx \
  --for=condition=Ready pod \
  --selector=app.kubernetes.io/component=controller \
  --timeout=180s

echo "[done] Cluster '${KIND_CLUSTER_NAME}' is ready. Ingress: http://localhost:8080"

