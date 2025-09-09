#!/usr/bin/env bash
set -euo pipefail

# Port-forward the gateway service to localhost for quick testing

NS="${NS:-dev}"
SERVICE="${SERVICE:-gateway}"
LOCAL_PORT="${LOCAL_PORT:-8080}"

echo "[check] ensuring service '${SERVICE}' exists in namespace '${NS}'"
kubectl get svc -n "${NS}" "${SERVICE}" >/dev/null

echo "[pf] forwarding localhost:${LOCAL_PORT} -> ${SERVICE}.svc:${NS}:80"
exec kubectl -n "${NS}" port-forward svc/"${SERVICE}" "${LOCAL_PORT}:80"

