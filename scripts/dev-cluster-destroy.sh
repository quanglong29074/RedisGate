#!/usr/bin/env bash
set -euo pipefail

KIND_CLUSTER_NAME="${KIND_CLUSTER_NAME:-redis-http-dev}"
REG_NAME="${REG_NAME:-kind-registry}"

echo "[kind] deleting cluster '${KIND_CLUSTER_NAME}' if exists"
if kind get clusters | grep -qx "${KIND_CLUSTER_NAME}"; then
  kind delete cluster --name "${KIND_CLUSTER_NAME}"
else
  echo "[kind] cluster not found, skipping"
fi

echo "[registry] stopping and removing local registry '${REG_NAME}' if exists"
if docker ps -aq -f name=^/${REG_NAME}$ >/dev/null; then
  docker rm -f "${REG_NAME}" >/dev/null || true
else
  echo "[registry] not found, skipping"
fi

echo "[done] Local dev environment cleaned up"

