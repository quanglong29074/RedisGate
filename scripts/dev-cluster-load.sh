#!/usr/bin/env bash
set -euo pipefail

# Build local images and load them into the Kind cluster.
# Defaults to only building 'gateway' to avoid failing on missing operator code.

KIND_CLUSTER_NAME="${KIND_CLUSTER_NAME:-redis-http-dev}"
COMPONENTS="${COMPONENTS:-gateway}"
TAG="${TAG:-dev}"
IMAGE_PREFIX="${IMAGE_PREFIX:-local}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

require_cmd() {
  command -v "$1" >/dev/null 2>&1 || { echo "Error: '$1' is required but not installed" >&2; exit 1; }
}

echo "[check] prerequisites"
require_cmd docker
require_cmd kind

build_and_load() {
  local comp="$1"
  local comp_dir="${ROOT_DIR}/${comp}"
  local dockerfile="${comp_dir}/Dockerfile"
  local image_ref="${IMAGE_PREFIX}/${comp}:${TAG}"

  if [[ ! -f "${dockerfile}" ]]; then
    echo "[skip] ${comp}: no Dockerfile at ${dockerfile}"
    return 0
  fi

  echo "[build] ${comp} -> ${image_ref}"
  docker build -t "${image_ref}" -f "${dockerfile}" "${comp_dir}"

  echo "[kind] loading image into cluster '${KIND_CLUSTER_NAME}'"
  kind load docker-image --name "${KIND_CLUSTER_NAME}" "${image_ref}"
}

for c in ${COMPONENTS}; do
  build_and_load "${c}"
done

echo "[done] Loaded components: ${COMPONENTS} (tag=${TAG}) into Kind"

