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

1.  A Developer create a API key to manage/interact with all hist instances in k8s cluster ( authenticates by JWT token )
2.  A Developer create a Redis instance in k8s cluster ( authenticates by JWT token ) , each instance have diffrence domain name
3.  A client (e.g., a Cloudflare Worker) sends an HTTP request to the **Shared HTTP Gateway** (Data Plane).
4.  The Gateway authenticates the request using the API key, identifies the target Redis instance from the URL, and translates the HTTP call into a native Redis command.
5.  The command is sent to the appropriate Redis instance over the internal cluster network.




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


### Method Override

To facilitate easy testing directly from a web browser, any `GET` request can simulate other methods (`POST`, `DELETE`, etc.) by using the `method` query parameter.
* **Rule:** Add `?method=METHOD_NAME` to a `GET` request's URL.
* **Arguments:** For methods that require a body (like `SET`), pass the data as additional query parameters (e.g., `&value=some_value`, `&field=some_field`).

**Example:**

The standard `POST` request to set a key:
```bash
curl -X POST http://localhost:8080/instances/user-session-cache/keys/user:123 \
  -H "Authorization: Bearer $API_KEY" \
  -H "Content-Type: application/json" \
  -d '{"value": "john_doe"}'
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
