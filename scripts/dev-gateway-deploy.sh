#!/usr/bin/env bash
set -euo pipefail

# Deploy the gateway into the dev namespace.
# Expects the image to be already built and kind-loaded, default: local/gateway:dev

NS="${NS:-dev}"
IMAGE="${IMAGE:-local/gateway:dev}"
REPLICAS="${REPLICAS:-1}"

kubectl get ns "${NS}" >/dev/null 2>&1 || kubectl create ns "${NS}"

cat <<EOF | kubectl apply -n "${NS}" -f -
apiVersion: apps/v1
kind: Deployment
metadata:
  name: gateway
  labels: { app: gateway }
spec:
  replicas: ${REPLICAS}
  selector:
    matchLabels: { app: gateway }
  template:
    metadata:
      labels: { app: gateway }
    spec:
      containers:
        - name: gateway
          image: ${IMAGE}
          imagePullPolicy: IfNotPresent
          ports:
            - name: http
              containerPort: 8080
          livenessProbe:
            httpGet:
              path: /
              port: 8080
            initialDelaySeconds: 5
            periodSeconds: 10
          readinessProbe:
            httpGet:
              path: /
              port: 8080
            initialDelaySeconds: 3
            periodSeconds: 5
---
apiVersion: v1
kind: Service
metadata:
  name: gateway
  labels: { app: gateway }
spec:
  type: ClusterIP
  selector: { app: gateway }
  ports:
    - name: http
      port: 80
      targetPort: 8080
---
apiVersion: networking.k8s.io/v1
kind: Ingress
metadata:
  name: gateway
  annotations:
    nginx.ingress.kubernetes.io/rewrite-target: /
spec:
  ingressClassName: nginx
  rules:
    - host: gateway.localdev.me
      http:
        paths:
          - path: /
            pathType: Prefix
            backend:
              service:
                name: gateway
                port:
                  number: 80
EOF

echo "[wait] waiting for deployment rollout"
kubectl -n "${NS}" rollout status deployment/gateway --timeout=180s

echo "[done] Gateway deployed in namespace '${NS}'. Ingress: http://gateway.localdev.me:8080/"

