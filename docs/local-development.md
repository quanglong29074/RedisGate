# Local Development with Kind

This guide describes how to spin up a local Kubernetes environment for developing the Redis-over-HTTP Cloud project using Kind, a local registry, and ingress-nginx.

## Prerequisites

- Docker
- kind
- kubectl

## Quickstart

1. Create the Kind cluster with local registry and ingress:
   - `scripts/dev-cluster-create.sh`
2. Build and load local images (default only gateway):
   - `scripts/dev-cluster-load.sh`
3. Deploy Redis test instances:
   - `scripts/dev-redis-deploy.sh`
4. Deploy the gateway (uses image `local/gateway:dev` by default):
   - `scripts/dev-gateway-deploy.sh`
5. Access the gateway:
   - Ingress: `http://gateway.localdev.me:8080/`
   - Or port-forward: `scripts/dev-port-forward.sh` then open `http://localhost:8080/`

## Commands Reference

- `scripts/dev-cluster-create.sh` — Creates cluster, local registry, installs ingress-nginx
- `scripts/dev-cluster-destroy.sh` — Deletes the cluster and local registry
- `scripts/dev-cluster-load.sh` — Builds and `kind load` images into cluster
  - Env: `COMPONENTS="gateway [operator]"`, `TAG=dev`, `IMAGE_PREFIX=local`
- `scripts/dev-redis-deploy.sh` — Deploys two Redis instances (`redis-a`, `redis-b`)
- `scripts/dev-gateway-deploy.sh` — Deploys gateway `Deployment`, `Service`, and `Ingress`
- `scripts/dev-port-forward.sh` — Port-forwards gateway service to `localhost:8080`

## Troubleshooting

- Ingress not reachable:
  - Ensure ingress-nginx controller is Ready: `kubectl -n ingress-nginx get pods`
  - Confirm Kind port mappings (80->8080, 443->8443) in `kind-config.yaml`
- Image not found:
  - Re-run `scripts/dev-cluster-load.sh` and confirm image name in gateway Deployment matches (`local/gateway:dev`)
- Port-forward fails:
  - Confirm gateway service exists: `kubectl -n dev get svc gateway`

