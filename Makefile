NS ?= dev
KIND_CLUSTER_NAME ?= redis-http-dev
IMAGE ?= local/gateway:dev
IMAGE_PREFIX ?= local
COMPONENTS ?= gateway
TAG ?= dev
REPLICAS ?= 1
SERVICE ?= gateway
LOCAL_PORT ?= 8080

.PHONY: cluster-up cluster-down load-images deploy-redis deploy-gateway port-forward dev-up dev-down smoke-test help

help:
	@echo "Available targets:"
	@echo "  make cluster-up      # Create Kind cluster + local registry + ingress"
	@echo "  make cluster-down    # Delete Kind cluster and local registry"
	@echo "  make load-images     # Build and kind-load images (COMPONENTS, TAG, IMAGE_PREFIX)"
	@echo "  make deploy-redis    # Deploy test Redis instances to namespace ($(NS))"
	@echo "  make deploy-gateway  # Deploy gateway image ($(IMAGE)) to namespace ($(NS))"
	@echo "  make port-forward    # Port-forward gateway service to localhost:$(LOCAL_PORT)"
	@echo "  make dev-up          # cluster-up + load-images + deploy-redis + deploy-gateway"
	@echo "  make dev-down        # Tear down local dev cluster"
	@echo "  make smoke-test      # Quick HTTP reachability check via ingress or localhost"

cluster-up:
	bash scripts/dev-cluster-create.sh

cluster-down:
	bash scripts/dev-cluster-destroy.sh

load-images:
	COMPONENTS="$(COMPONENTS)" TAG="$(TAG)" IMAGE_PREFIX="$(IMAGE_PREFIX)" \
		bash scripts/dev-cluster-load.sh

deploy-redis:
	NS="$(NS)" bash scripts/dev-redis-deploy.sh

deploy-gateway:
	NS="$(NS)" IMAGE="$(IMAGE)" REPLICAS="$(REPLICAS)" bash scripts/dev-gateway-deploy.sh

port-forward:
	NS="$(NS)" SERVICE="$(SERVICE)" LOCAL_PORT="$(LOCAL_PORT)" bash scripts/dev-port-forward.sh

dev-up: cluster-up load-images deploy-redis deploy-gateway

dev-down: cluster-down

smoke-test:
	@echo "[smoke] testing ingress http://gateway.localdev.me:8080/ (fallback to localhost:$(LOCAL_PORT))"
	@set -e; \
	  if curl -fsS -m 3 http://gateway.localdev.me:8080/ >/dev/null; then \
	    echo "[ok] ingress reachable"; \
	  else \
	    echo "[warn] ingress not reachable, trying localhost:$(LOCAL_PORT)"; \
	    curl -fsS -m 3 http://localhost:$(LOCAL_PORT)/ >/dev/null; \
	    echo "[ok] localhost port-forward reachable"; \
	  fi

