# Redis Development with Minikube

This directory contains scripts and configurations for running Redis in a local Minikube environment for development purposes.

## 📁 Directory Structure

```
minikube/
├── config/
│   ├── redis-config.yaml       # Redis server configuration
│   └── minikube-config.yaml    # Minikube cluster settings
├── scripts/
│   ├── minikube-setup.sh       # Setup minikube cluster
│   ├── redis-deploy.sh         # Deploy Redis server
│   └── port-forward.sh         # Port forward Redis to localhost
├── templates/
│   ├── redis-deployment.yaml   # Kubernetes Redis deployment
│   └── redis-service.yaml      # Kubernetes Redis service
└── README.md                   # This file
```

## 🚀 Quick Start

### Prerequisites

Make sure you have the following tools installed:
- [Minikube](https://minikube.sigs.k8s.io/docs/start/)
- [kubectl](https://kubernetes.io/docs/tasks/tools/)
- [Docker](https://docs.docker.com/get-docker/)
- [redis-cli](https://redis.io/docs/getting-started/installation/) (optional, for testing)

### 1. Setup Minikube Cluster

```bash
cd minikube/scripts
chmod +x *.sh
./minikube-setup.sh
```

This will:
- Create a minikube cluster named `redis-dev`
- Enable necessary addons (dashboard, ingress, metrics-server)
- Create `redis-dev` namespace
- Configure kubectl context

### 2. Deploy Redis Server

```bash
./redis-deploy.sh
```

This will:
- Create ConfigMap with Redis configuration
- Deploy Redis with custom config
- Create ClusterIP and NodePort services
- Wait for deployment to be ready

### 3. Access Redis

#### Option 1: Port Forward (Recommended)
```bash
./port-forward.sh
```

Then connect from another terminal:
```bash
redis-cli -h localhost -p 6379
```

#### Option 2: NodePort Access
```bash
minikube service redis-server-nodeport -p redis-dev --url
```

#### Option 3: Minikube IP + NodePort
```bash
minikube ip -p redis-dev  # Get cluster IP
redis-cli -h <MINIKUBE_IP> -p 30379
```

## 🔧 Configuration

### Modify Redis Configuration

Edit `config/redis-config.yaml` to customize Redis settings:
- Memory limits
- Persistence settings
- Security options
- Logging levels

After modifying, redeploy:
```bash
./redis-deploy.sh
```

### Modify Cluster Configuration

Edit `config/minikube-config.yaml` to customize cluster:
- CPU and memory allocation
- Kubernetes version
- Enabled addons
- Namespace settings

After modifying, recreate cluster:
```bash
minikube delete -p redis-dev
./minikube-setup.sh
```

## 📊 Monitoring

### Kubernetes Dashboard
```bash
minikube dashboard -p redis-dev
```

### Check Redis Status
```bash
kubectl get pods -n redis-dev
kubectl logs -f deployment/redis-server -n redis-dev
```

### Redis Info
```bash
redis-cli -h localhost -p 6379 info
```

## 🧹 Cleanup

### Stop Port Forward
Press `Ctrl+C` in the terminal running port-forward

### Delete Redis Deployment
```bash
kubectl delete -f ../templates/redis-deployment.yaml -n redis-dev
kubectl delete -f ../templates/redis-service.yaml -n redis-dev
```

### Delete Entire Cluster
```bash
minikube delete -p redis-dev
```

## 🔍 Troubleshooting

### Check Cluster Status
```bash
minikube status -p redis-dev
kubectl cluster-info --context redis-dev
```

### Check Redis Pod Logs
```bash
kubectl logs -f deployment/redis-server -n redis-dev
```

### Test Redis Connection
```bash
kubectl exec -it deployment/redis-server -n redis-dev -- redis-cli ping
```

### Port Forward Issues
```bash
# Kill existing port forwards
pkill -f "kubectl.*port-forward"
# Check if service exists
kubectl get svc -n redis-dev
```

## 📝 Development Tips

1. **Persistent Data**: Current setup uses `emptyDir` for Redis data. For persistent data, modify the deployment to use PVC.

2. **Multiple Redis Instances**: Copy and modify the templates with different names to run multiple Redis instances.

3. **Configuration Changes**: After changing `redis-config.yaml`, delete the ConfigMap and redeploy:
   ```bash
   kubectl delete configmap redis-config -n redis-dev
   ./redis-deploy.sh
   ```

4. **Resource Monitoring**: Use `kubectl top pods -n redis-dev` to monitor resource usage.

5. **Network Testing**: Use `kubectl run -it --rm debug --image=busybox --restart=Never -- sh` to debug network connectivity.

## 🎯 Next Steps

- Add Redis Cluster configuration
- Implement backup/restore scripts
- Add monitoring with Prometheus
- Create Helm charts for easier deployment
- Add CI/CD integration
