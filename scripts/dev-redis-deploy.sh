#!/usr/bin/env bash
set -euo pipefail

# Deploy two test Redis instances in the dev namespace

NS="${NS:-dev}"

kubectl get ns "${NS}" >/dev/null 2>&1 || kubectl create ns "${NS}"

cat <<'EOF' | kubectl apply -n "${NS}" -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: redis-a
  labels: { app: redis-a }
spec:
  replicas: 1
  selector:
    matchLabels: { app: redis-a }
  template:
    metadata:
      labels: { app: redis-a }
    spec:
      containers:
        - name: redis
          image: redis:7-alpine
          imagePullPolicy: IfNotPresent
          ports:
            - containerPort: 6379
          readinessProbe:
            tcpSocket: { port: 6379 }
            initialDelaySeconds: 2
            periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: redis-a
  labels: { app: redis-a }
spec:
  type: ClusterIP
  selector: { app: redis-a }
  ports:
    - name: redis
      port: 6379
      targetPort: 6379
---
apiVersion: apps/v1
kind: Deployment
metadata:
  name: redis-b
  labels: { app: redis-b }
spec:
  replicas: 1
  selector:
    matchLabels: { app: redis-b }
  template:
    metadata:
      labels: { app: redis-b }
    spec:
      containers:
        - name: redis
          image: redis:7-alpine
          imagePullPolicy: IfNotPresent
          ports:
            - containerPort: 6379
          readinessProbe:
            tcpSocket: { port: 6379 }
            initialDelaySeconds: 2
            periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: redis-b
  labels: { app: redis-b }
spec:
  type: ClusterIP
  selector: { app: redis-b }
  ports:
    - name: redis
      port: 6379
      targetPort: 6379
EOF

echo "[done] Redis instances 'redis-a' and 'redis-b' deployed in namespace '${NS}'"

