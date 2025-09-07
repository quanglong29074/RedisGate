# KubeRedis HTTP Gateway

[](https://www.google.com/search?q=https://github.com/your-repo/your-project/actions)
[](https://opensource.org/licenses/MIT)
[](https://www.google.com/search?q=https://github.com/your-repo/your-project/releases)

A cloud-native solution for providing Redis-as-a-Service on Kubernetes, accessible via a secure, high-performance, shared RESTful API. Designed for modern serverless and edge environments where direct TCP connections are restricted.

## 🎯 Problem Statement

Modern serverless and edge computing platforms (e.g., Cloudflare Workers, Vercel Edge Functions) often prohibit direct TCP socket connections. This creates a significant barrier for applications needing to leverage high-performance in-memory data stores like Redis for caching, session management, or real-time data processing. **KubeRedis HTTP Gateway** bridges this gap.

## ✨ Features

  * **Declarative Provisioning:** Create and manage dedicated Redis instances using a simple Kubernetes Custom Resource (`RedisHttpInstance`).
  * **HTTP/S Access:** Interact with Redis using a standard RESTful API, eliminating the need for TCP clients.
  * **Centralized High-Performance Gateway:** A single, multi-tenant gateway written in **Rust** handles all traffic, offering high concurrency and low latency through a non-blocking architecture.
  * **Automated Lifecycle Management:** A Kubernetes Operator handles the entire lifecycle of Redis instances, from provisioning to decommissioning.
  * **Secure by Design:** Each instance is isolated and protected by a unique, auto-generated API key.
  * **Cloud-Native:** Built from the ground up to leverage the power and scalability of Kubernetes.

## 🏗️ System Architecture

The system consists of two primary components: a **Control Plane** (the Operator) and a **Data Plane** (the shared Gateway).

1.  A developer defines a `RedisHttpInstance` YAML file.
2.  `kubectl apply` sends the manifest to the Kubernetes API Server.
3.  The **Kubernetes Operator** (Control Plane) detects the new resource and provisions the required components: a Redis `StatefulSet`, a headless `Service`, and a `Secret` containing a unique API key.
4.  A client (e.g., a Cloudflare Worker) sends an HTTP request to the **Shared HTTP Gateway** (Data Plane).
5.  The Gateway authenticates the request using the API key, identifies the target Redis instance from the URL, and translates the HTTP call into a native Redis command.
6.  The command is sent to the appropriate Redis instance over the internal cluster network.

 *(Conceptual Diagram)*

-----

## 🚀 Getting Started

### Prerequisites

  * A running Kubernetes cluster (e.g., Minikube, Kind, GKE, EKS).
  * `kubectl` configured to connect to your cluster.

### 1\. Installation

First, install the Custom Resource Definition (CRD) and deploy the Operator and the shared Gateway to your cluster.

```bash
# 1. Apply the CRD so Kubernetes understands what a 'RedisHttpInstance' is
kubectl apply -f https://raw.githubusercontent.com/your-repo/your-project/main/config/crd.yaml

# 2. Deploy the Kubernetes Operator
kubectl apply -f https://raw.githubusercontent.com/your-repo/your-project/main/config/operator-deployment.yaml

# 3. Deploy the shared HTTP Gateway
kubectl apply -f https://raw.githubusercontent.com/your-repo/your-project/main/config/gateway-deployment.yaml

# 4. (Optional) Expose the gateway via an Ingress or LoadBalancer service
# For this example, we will use port-forwarding.
```

### 2\. Provision a Redis Instance

Create a YAML file named `my-cache.yaml` to declare your new Redis instance.

```yaml
# my-cache.yaml
apiVersion: "cache.yourcompany.com/v1alpha1"
kind: "RedisHttpInstance"
metadata:
  name: "user-session-cache"
spec:
  capacity: "1Gi"
  storageClassName: "standard" # Use a storage class available in your cluster
```

Apply it to the cluster:

```bash
kubectl apply -f my-cache.yaml
```

The operator will now provision a Redis instance named `user-session-cache`.

### 3\. Retrieve Credentials

The operator automatically creates a secret containing the API key for your new instance. The secret will be named `{instance-name}-auth`.

```bash
# Retrieve the API key and decode it
API_KEY=$(kubectl get secret user-session-cache-auth \
  -o jsonpath='{.data.api_key}' | base64 --decode)

echo "Your API Key: $API_KEY"
```

### 4\. Interact with Redis via HTTP

Forward a local port to the gateway service to test it.

```bash
# In a new terminal
kubectl port-forward svc/kuberedis-http-gateway 8080:80
```

Now, use `curl` to interact with your Redis instance.

```bash
# Set a key
curl -X POST http://localhost:8080/instances/user-session-cache/keys/user:123 \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"value": "{\"name\": \"John Doe\", \"active\": true}"}'

# Response: {"status":"OK"}

# Get the key back
curl http://localhost:8080/instances/user-session-cache/keys/user:123 \
  -H "Authorization: Bearer $API_KEY"

# Response: {"key":"user:123","value":"{\"name\": \"John Doe\", \"active\": true}"}
```

-----

## 📖 API Specification

**Authentication:** All endpoints require an `Authorization: Bearer <API_KEY>` header.

**Base Path:** `/instances/{instance_name}`

| Redis Command | HTTP Method | Endpoint                       | Request Body (JSON)             | Success Response (200 OK)                          |
| :------------ | :---------- | :----------------------------- | :------------------------------ | :------------------------------------------------- |
| `SET`         | `POST`      | `/keys/{key}`                  | `{"value": "...", "ttl_seconds": 60}` | `{"status": "OK"}`                                 |
| `GET`         | `GET`       | `/keys/{key}`                  | -                               | `{"key": "mykey", "value": "some_value"}`          |
| `DEL`         | `DELETE`    | `/keys/{key}`                  | -                               | `{"deleted": 1}`                                   |
| `HSET`        | `POST`      | `/hashes/{hash}`               | `{"field": "f1", "value": "v1"}` | `{"status": "OK"}`                                 |
| `HGET`        | `GET`       | `/hashes/{hash}/{field}`       | -                               | `{"hash": "h", "field": "f1", "value": "v1"}`      |
| Raw Command   | `POST`      | `/raw-command`                 | `{"command": "XADD", "args": ["s", "*", "f1", "v1"]}` | `{"result": "1725700000000-0"}` |

-----

## 🗺️ Roadmap

Our vision is to evolve this into a robust, enterprise-ready data platform solution.

  * **Q4 2025: Production Hardening**

      * [ ] **Dynamic Configuration:** Fully automate gateway routing and secret management by having it watch Kubernetes resources directly.
      * [ ] **Observability:** Integrate structured logging (`tracing`), Prometheus metrics, and OpenTelemetry for distributed tracing.
      * [ ] **Expanded API:** Support for more complex Redis commands, transactions (`MULTI`/`EXEC`), and Pub/Sub.

  * **Q1 2026: Scalability & Usability**

      * [ ] **High Availability:** Operator support for Redis Sentinel or Redis Cluster configurations.
      * [ ] **Enhanced Security:** Implement mTLS between the gateway and Redis backends.
      * [ ] **Dashboard/UI:** A simple web interface for users to view their provisioned instances and manage API keys.

## 🤝 Contributing

Contributions are welcome\! Please refer to the `CONTRIBUTING.md` file for guidelines on how to submit issues, and pull requests.

## 📄 License

This project is licensed under the **MIT License**. See the [LICENSE](https://www.google.com/search?q=LICENSE) file for details.
