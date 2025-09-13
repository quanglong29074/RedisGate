# RedisGate

[](https://www.google.com/search?q=https://github.com/your-repo/your-project/actions)
[](https://opensource.org/licenses/MIT)
[](https://www.google.com/search?q=https://github.com/your-repo/your-project/releases)

A cloud-native solution for providing Redis-as-a-Service on Kubernetes, accessible via a secure, high-performance, shared RESTful API. This repository contains the management API and control plane components. Designed for modern serverless and edge environments where direct TCP connections are restricted.

## 🚀 Quick Start

### Development Setup

This project provides the management API and control plane for RedisGate. The development environment includes PostgreSQL for metadata storage, but actual Redis instances are managed by the Kubernetes operator.

```bash
# One-time setup (installs all dependencies)
./setup-dev.sh

# Start development services (PostgreSQL)
make dev

# Start Minikube and deploy
make deploy

# Full development setup
make dev-full
```

For detailed development setup instructions, see [DEVELOPMENT.md](DEVELOPMENT.md).

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




## 📖 REST API

REST API enables you to access your Redis database using HTTP requests.

### Get Started

**Authentication:** All endpoints require an `Authorization: Bearer <TOKEN>` header.

Copy your endpoint URL and token. Send an HTTP request to the provided URL by adding an `Authorization: Bearer $TOKEN` header like below:

```shell
curl https://your-redis-instance.yourdomain.com/set/foo/bar \
  -H "Authorization: Bearer your_api_token"
```

The above script executes a `SET foo bar` command. It will return a JSON response:

```json
{ "result": "OK" }
```

You can also set the token as `_token` request parameter:

```shell
curl https://your-redis-instance.yourdomain.com/set/foo/bar?_token=your_api_token
```

### API Semantics

The REST API follows the same convention with [Redis Protocol](https://redis.io/commands). Give the command name and parameters in the same order as Redis protocol by separating them with a `/`.

```shell
curl REST_URL/COMMAND/arg1/arg2/../argN
```

Here are some examples:

* `SET foo bar` -> `REST_URL/set/foo/bar`
* `SET foo bar EX 100` -> `REST_URL/set/foo/bar/EX/100`
* `GET foo` -> `REST_URL/get/foo`
* `MGET foo1 foo2 foo3` -> `REST_URL/mget/foo1/foo2/foo3`
* `HGET employee:23381 salary` -> `REST_URL/hget/employee:23381/salary`
* `ZADD teams 100 team-x 90 team-y` -> `REST_URL/zadd/teams/100/team-x/90/team-y`

#### JSON or Binary Value

To post a JSON or a binary value, you can use an HTTP POST request and set value as the request body:

```shell
curl -X POST -d '$VALUE' https://your-redis-instance.yourdomain.com/set/foo \
  -H "Authorization: Bearer your_api_token"
```

In the example above, `$VALUE` sent in request body is appended to the command as `REST_URL/set/foo/$VALUE`.

Please note that when making a POST request, the request body is appended as the last parameter of the Redis command. If there are additional parameters in the Redis command after the value, you should include them as query parameters in the request:

```shell
curl -X POST -d '$VALUE' https://your-redis-instance.yourdomain.com/set/foo?EX=100 \
  -H "Authorization: Bearer your_api_token"
```

Above command is equivalent to `REST_URL/set/foo/$VALUE/EX/100`.

#### POST Command in Body

Alternatively, you can send the whole command in the request body as a single JSON array. Array's first element must be the command name and command parameters should be appended next to each other in the same order as Redis protocol.

```shell
curl -X POST -d '[COMMAND, ARG1, ARG2,.., ARGN]' REST_URL
```

For example, Redis command `SET foo bar EX 100` can be sent inside the request body as:

```shell
curl -X POST -d '["SET", "foo", "bar", "EX", 100]' https://your-redis-instance.yourdomain.com \
  -H "Authorization: Bearer your_api_token"
```

### HTTP Codes

* `200 OK`: When request is accepted and successfully executed.
* `400 Bad Request`: When there's a syntax error, an invalid/unsupported command is sent or command execution fails.
* `401 Unauthorized`: When authentication fails; auth token is missing or invalid.
* `405 Method Not Allowed`: When an unsupported HTTP method is used. Only `HEAD`, `GET`, `POST` and `PUT` methods are allowed.

### Response

REST API returns a JSON response by default. When command execution is successful, response JSON will have a single `result` field and its value will contain the Redis response. It can be either;

* a `null` value

```json
{ "result": null }
```

* an integer

```json
{ "result": 137 }
```

* a string

```json
{ "result": "value" }
```

* an array value:

```json
{ "result": ["value1", null, "value2"] }
```

If command is rejected or fails, response JSON will have a single `error` field with a string value explaining the failure:

```json
{"error":"WRONGPASS invalid password"}

{"error":"ERR wrong number of arguments for 'get' command"}
```

#### Base64 Encoded Responses

If the response contains an invalid utf-8 character, it will be replaced with a � (Replacement character U+FFFD). This can happen when you are using binary operations like `BITOP NOT` etc.

If you prefer the raw response in base64 format, you can achieve this by setting the `Upstash-Encoding` header to `base64`. In this case, all strings in the response will be base64 encoded, except for the "OK" response.

```shell
curl https://your-redis-instance.yourdomain.com/SET/foo/bar \
  -H "Authorization: Bearer your_api_token" \
  -H "Upstash-Encoding: base64"

# {"result":"OK"}

curl https://your-redis-instance.yourdomain.com/GET/foo \
  -H "Authorization: Bearer your_api_token" \
  -H "Upstash-Encoding: base64"

# {"result":"YmFy"}
```

#### RESP2 Format Responses

REST API returns a JSON response by default and the response content type is set to `application/json`.

If you prefer the binary response in RESP2 format, you can achieve this by setting the `Upstash-Response-Format` header to `resp2`. In this case, the response content type is set to `application/octet-stream` and the raw response is returned as binary similar to a TCP-based Redis client.

The default value for this option is `json`. Any format other than `json` and `resp2` is not allowed and will result in a HTTP 400 Bad Request.

This option is not applicable to `/multi-exec` transactions endpoint, as it only returns response in JSON format. Additionally, setting the `Upstash-Encoding` header to `base64` is not permitted when the `Upstash-Response-Format` is set to `resp2` and will result in a HTTP 400 Bad Request.

```shell
curl https://your-redis-instance.yourdomain.com/SET/foo/bar \
  -H "Authorization: Bearer your_api_token" \
  -H "Upstash-Response-Format: resp2"

# +OK\r\n

curl https://your-redis-instance.yourdomain.com/GET/foo \
  -H "Authorization: Bearer your_api_token" \
  -H "Upstash-Response-Format: resp2"

# $3\r\nbar\r\n
```

### Pipelining

REST API provides support for command pipelining, allowing you to send multiple commands as a batch instead of sending them individually and waiting for responses. With the pipeline API, you can include several commands in a single HTTP request, and the response will be a JSON array. Each item in the response array corresponds to the result of a command in the same order as they were included in the pipeline.

API endpoint for command pipelining is `/pipeline`. Pipelined commands should be send as a two dimensional JSON array in the request body, each row containing name of the command and its arguments.

**Request syntax**:

```shell
curl -X POST https://your-redis-instance.yourdomain.com/pipeline \
  -H "Authorization: Bearer $TOKEN" \
  -d '
    [
      ["CMD_A", "arg0", "arg1", ..., "argN"],
      ["CMD_B", "arg0", "arg1", ..., "argM"],
      ...
    ]
    '
```

**Response syntax**:

```json
[{"result":"RESPONSE_A"},{"result":"RESPONSE_B"},{"error":"ERR ..."}, ...]
```

> **Note:** Execution of the pipeline is *not atomic*. Even though each command in the pipeline will be executed in order, commands sent by other clients can interleave with the pipeline. Use [transactions](#transactions) API instead if you need atomicity.

For example you can write the `curl` command below to send following Redis commands using pipeline:

```redis
SET key1 valuex
SETEX key2 13 valuez
INCR key1
ZADD myset 11 item1 22 item2
```

```shell
curl -X POST https://your-redis-instance.yourdomain.com/pipeline \
  -H "Authorization: Bearer your_api_token" \
  -d '
    [
      ["SET", "key1", "valuex"],
      ["SETEX", "key2", 13, "valuez"],
      ["INCR", "key1"],
      ["ZADD", "myset", 11, "item1", 22, "item2"]
    ]
    '
```

And pipeline response will be:

```json
[
  { "result": "OK" },
  { "result": "OK" },
  { "error": "ERR value is not an int or out of range" },
  { "result": 2 }
]
```

You can use pipelining when;

* You need more throughput, since pipelining saves from multiple round-trip times. (*But beware that latency of each command in the pipeline will be equal to the total latency of the whole pipeline.*)
* Your commands are independent of each other, response of a former command is not needed to submit a subsequent command.

### Transactions

REST API supports transactions to execute multiple commands atomically. With transactions API, several commands are sent using a single HTTP request, and a single JSON array response is returned. Each item in the response array corresponds to the command in the same order within the transaction.

API endpoint for transaction is `/multi-exec`. Transaction commands should be send as a two dimensional JSON array in the request body, each row containing name of the command and its arguments.

**Request syntax**:

```shell
curl -X POST https://your-redis-instance.yourdomain.com/multi-exec \
  -H "Authorization: Bearer $TOKEN" \
  -d '
    [
      ["CMD_A", "arg0", "arg1", ..., "argN"],
      ["CMD_B", "arg0", "arg1", ..., "argM"],
      ...
    ]
    '
```

**Response syntax**:

In case when transaction is successful, multiple responses corresponding to each command is returned in json as follows:

```json
[{"result":"RESPONSE_A"},{"result":"RESPONSE_B"},{"error":"ERR ..."}, ...]
```

If transaction is discarded as a whole, a single error is returned in json as follows:

```json
{ "error": "ERR ..." }
```

A transaction might be discarded in following cases:

* There is a syntax error on the transaction request.
* At least one of the commands is unsupported.
* At least one of the commands exceeds the max request size.
* At least one of the commands exceeds the daily request limit.

Note that a command may still fail even if it is a supported and valid command. In that case, all commands will be executed. Redis will not stop the processing of commands. This is to provide same semantics with Redis when there are [errors inside a transaction](https://redis.io/docs/manual/transactions/#errors-inside-a-transaction).

**Example**:

You can write the `curl` command below to send following Redis commands using REST transaction API:

```
MULTI
SET key1 valuex
SETEX key2 13 valuez
INCR key1
ZADD myset 11 item1 22 item2
EXEC
```

```shell
curl -X POST https://your-redis-instance.yourdomain.com/multi-exec \
  -H "Authorization: Bearer your_api_token" \
  -d '
    [
      ["SET", "key1", "valuex"],
      ["SETEX", "key2", 13, "valuez"],
      ["INCR", "key1"],
      ["ZADD", "myset", 11, "item1", 22, "item2"]
    ]
    '
```

And transaction response will be:

```json
[
  { "result": "OK" },
  { "result": "OK" },
  { "error": "ERR value is not an int or out of range" },
  { "result": 2 }
]
```

### Monitor Command

REST API provides Redis [`MONITOR`](https://redis.io/docs/latest/commands/monitor/) command using [Server Send Events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events) mechanism. API endpoint is `/monitor`.

```shell
curl -X POST https://your-redis-instance.yourdomain.com/monitor \
  -H "Authorization: Bearer your_api_token" \
  -H "Accept:text/event-stream"
```

This request will listen for Redis monitor events and incoming data will be received as:

```
data: "OK"
data: 1721284005.242090 [0 0.0.0.0:0] "GET" "k"
data: 1721284008.663811 [0 0.0.0.0:0] "SET" "k" "v"
data: 1721284025.561585 [0 0.0.0.0:0] "DBSIZE"
data: 1721284030.601034 [0 0.0.0.0:0] "KEYS" "*"
```

### Subscribe & Publish Commands

Similar to `MONITOR` command, REST API provides Redis [`SUBSCRIBE`](https://redis.io/docs/latest/commands/subscribe/) and [`PUBLISH`](https://redis.io/docs/latest/commands/publish/) commands. The `SUBSCRIBE` endpoint works using [Server Send Events](https://developer.mozilla.org/en-US/docs/Web/API/Server-sent_events) mechanism. API endpoints are `/subscribe` and `/publish`

Following request will subscribe to a channel named `chat`:

```shell
curl -X POST https://your-redis-instance.yourdomain.com/subscribe/chat \
  -H "Authorization: Bearer your_api_token" \
  -H "Accept:text/event-stream"
```

Following request will publish to a channel named `chat`:

```shell
curl -X POST https://your-redis-instance.yourdomain.com/publish/chat/hello \
  -H "Authorization: Bearer your_api_token"
```

The subscriber will receive incoming messages as:

```
data: subscribe,chat,1
data: message,chat,hello
data: message,chat,how are you today?
```

### Security and Authentication

You need to add a header to your API requests as `Authorization: Bearer $TOKEN` or set the token as a url parameter `_token=$TOKEN`.

```shell
curl -X POST https://your-redis-instance.yourdomain.com/info \
  -H "Authorization: Bearer your_api_token"
```

OR

```shell
curl -X POST https://your-redis-instance.yourdomain.com/info?_token=your_api_token
```

### REST - Redis API Compatibility

| Feature                                                       | REST Support? |                               Notes                               |
| ------------------------------------------------------------- | :-----------: | :---------------------------------------------------------------: |
| [String](https://redis.io/commands/?group=string)             |       ✅       |                                                                   |
| [Bitmap](https://redis.io/commands/?group=bitmap)             |       ✅       |                                                                   |
| [Hash](https://redis.io/commands/?group=hash)                 |       ✅       |                                                                   |
| [List](https://redis.io/commands/?group=list)                 |       ✅       | Blocking commands (BLPOP - BRPOP - BRPOPLPUSH) are not supported. |
| [Set](https://redis.io/commands/?group=set)                   |       ✅       |                                                                   |
| [SortedSet](https://redis.io/commands/?group=sorted_set)      |       ✅       |     Blocking commands (BZPOPMAX - BZPOPMIN) are not supported.    |
| [Geo](https://redis.io/commands/?group=geo)                   |       ✅       |                                                                   |
| [HyperLogLog](https://redis.io/commands/?group=hyperloglog)   |       ✅       |                                                                   |
| [Transactions](https://redis.io/commands/?group=transactions) |       ✅       |              WATCH/UNWATCH/DISCARD are not supported              |
| [Generic](https://redis.io/commands/?group=generic)           |       ✅       |                                                                   |
| [Server](https://redis.io/commands/?group=server)             |       ✅       |                                                                   |
| [Scripting](https://redis.io/commands/?group=scripting)       |       ✅       |                                                                   |
| [Pub/Sub](https://redis.io/commands/?group=pubsub)            |       ⚠️      |               Only PUBLISH and SUBSCRIBE are supported.              |
| [Connection](https://redis.io/commands/?group=connection)     |       ⚠️      |                 Only PING and ECHO are supported.                 |
| [JSON](https://redis.io/commands/?group=json)                 |       ✅       |                                                                   |
| [Streams](https://redis.io/commands/?group=stream)            |       ✅       |    Supported, except blocking versions of XREAD and XREADGROUP.   |
| [Cluster](https://redis.io/commands#cluster)                  |       ❌       |                                                                   |

### Redis Protocol vs REST API

#### REST API Pros

* If you want to access your Redis database from an environment like CloudFlare Workers, WebAssembly, Fastly Compute@Edge then you can not use Redis protocol as it is based on TCP. You can use REST API in those environments.

* REST API is request (HTTP) based where Redis protocol is connection based. If you are running serverless functions (AWS Lambda etc), you may need to manage the Redis client's connections. REST API does not have such an issue.

* Redis protocol requires Redis clients. On the other hand, REST API is accessible with any HTTP client.

#### Redis Protocol Pros

* If you have legacy code that relies on Redis clients, the Redis protocol allows you to utilize Redis without requiring any modifications to your code.

* By leveraging the Redis protocol, you can take advantage of the extensive Redis ecosystem. For instance, you can seamlessly integrate your Redis database as a session cache for your Express application.

### Cost and Pricing

Pricing is based on per command/request. The same pricing applies to your REST calls.

### Metrics and Monitoring

In the current version, we do not expose any metrics specific to API calls in the console. But the metrics of the database backing the API should give a good summary about the performance of your APIs.

## 🚀 CI/CD Pipeline

This project includes a comprehensive GitHub Actions CI/CD pipeline that automatically:

- **Code Quality**: Runs `cargo fmt` and `cargo clippy` for code formatting and linting
- **Testing**: Executes the full test suite with PostgreSQL service dependencies
- **Building**: Creates optimized release builds for multiple targets
- **Security**: Performs security audits using `cargo audit`
- **Docker**: Builds and validates Docker images
- **Artifacts**: Uploads release binaries for distribution

The pipeline runs on every push to `main` and `develop` branches, as well as on pull requests.

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
