# Redis HTTP API Documentation

RedisGate provides a comprehensive Redis HTTP API that allows you to interact with Redis instances through HTTP requests. All Redis operations are secured with API key authentication and scoped to your organization.

## Authentication

All Redis API requests require authentication using an API key. You can provide the API key in two ways:

1. **Authorization Header** (Recommended):
   ```bash
   Authorization: Bearer your-api-key-here
   ```

2. **Query Parameter**:
   ```bash
   ?_token=your-api-key-here
   ```

## Base URL Structure

All Redis API endpoints follow this pattern:
```
/redis/{instance_id}/{command}/...
```

Where `instance_id` is the UUID of your Redis instance.

## Supported Commands

### Basic String Operations

#### GET
Retrieve the value of a key.
```bash
GET /redis/{instance_id}/get/{key}
```

**Example:**
```bash
curl "http://localhost:8080/redis/123e4567-e89b-12d3-a456-426614174000/get/mykey" \
  -H "Authorization: Bearer your-api-key"
```

#### SET
Set a key to a value.
```bash
GET /redis/{instance_id}/set/{key}/{value}
```

**Example:**
```bash
curl "http://localhost:8080/redis/123e4567-e89b-12d3-a456-426614174000/set/mykey/myvalue" \
  -H "Authorization: Bearer your-api-key"
```

#### DEL
Delete a key.
```bash
GET /redis/{instance_id}/del/{key}
```

**Example:**
```bash
curl "http://localhost:8080/redis/123e4567-e89b-12d3-a456-426614174000/del/mykey" \
  -H "Authorization: Bearer your-api-key"
```

#### INCR
Increment a key's value by 1.
```bash
GET /redis/{instance_id}/incr/{key}
```

**Example:**
```bash
curl "http://localhost:8080/redis/123e4567-e89b-12d3-a456-426614174000/incr/counter" \
  -H "Authorization: Bearer your-api-key"
```

### Hash Operations

#### HSET
Set a field in a hash.
```bash
GET /redis/{instance_id}/hset/{key}/{field}/{value}
```

**Example:**
```bash
curl "http://localhost:8080/redis/123e4567-e89b-12d3-a456-426614174000/hset/user:1/name/john" \
  -H "Authorization: Bearer your-api-key"
```

#### HGET
Get a field from a hash.
```bash
GET /redis/{instance_id}/hget/{key}/{field}
```

**Example:**
```bash
curl "http://localhost:8080/redis/123e4567-e89b-12d3-a456-426614174000/hget/user:1/name" \
  -H "Authorization: Bearer your-api-key"
```

### List Operations

#### LPUSH
Push a value to the left (head) of a list.
```bash
GET /redis/{instance_id}/lpush/{key}/{value}
```

**Example:**
```bash
curl "http://localhost:8080/redis/123e4567-e89b-12d3-a456-426614174000/lpush/mylist/item1" \
  -H "Authorization: Bearer your-api-key"
```

#### LPOP
Pop a value from the left (head) of a list.
```bash
GET /redis/{instance_id}/lpop/{key}
```

**Example:**
```bash
curl "http://localhost:8080/redis/123e4567-e89b-12d3-a456-426614174000/lpop/mylist" \
  -H "Authorization: Bearer your-api-key"
```

### Utility Commands

#### PING
Test connectivity to the Redis instance.
```bash
GET /redis/{instance_id}/ping
```

**Example:**
```bash
curl "http://localhost:8080/redis/123e4567-e89b-12d3-a456-426614174000/ping" \
  -H "Authorization: Bearer your-api-key"
```

## Generic Command Endpoint

For advanced use cases, you can send any Redis command using the generic POST endpoint:

```bash
POST /redis/{instance_id}
Content-Type: application/json
```

**Request Body Format:**
```json
["COMMAND", "arg1", "arg2", "..."]
```

### Examples

**SET with expiration:**
```bash
curl -X POST "http://localhost:8080/redis/123e4567-e89b-12d3-a456-426614174000" \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '["SETEX", "session:123", "3600", "user_data"]'
```

**Get multiple keys:**
```bash
curl -X POST "http://localhost:8080/redis/123e4567-e89b-12d3-a456-426614174000" \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '["MGET", "key1", "key2", "key3"]'
```

**Hash operations:**
```bash
curl -X POST "http://localhost:8080/redis/123e4567-e89b-12d3-a456-426614174000" \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '["HGETALL", "user:1"]'
```

**List operations:**
```bash
curl -X POST "http://localhost:8080/redis/123e4567-e89b-12d3-a456-426614174000" \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '["LRANGE", "mylist", "0", "-1"]'
```

**Set operations:**
```bash
curl -X POST "http://localhost:8080/redis/123e4567-e89b-12d3-a456-426614174000" \
  -H "Authorization: Bearer your-api-key" \
  -H "Content-Type: application/json" \
  -d '["SADD", "myset", "member1", "member2"]'
```

## Supported Commands via Generic Endpoint

The generic endpoint supports a comprehensive set of Redis commands:

### String Commands
- `GET`, `SET`, `DEL`, `EXISTS`, `INCR`, `DECR`, `APPEND`, `STRLEN`
- `EXPIRE`, `TTL` (key expiration)

### Hash Commands
- `HSET`, `HGET`, `HDEL`, `HEXISTS`, `HGETALL`, `HKEYS`, `HVALS`

### List Commands
- `LPUSH`, `RPUSH`, `LPOP`, `RPOP`, `LLEN`, `LRANGE`

### Set Commands
- `SADD`, `SREM`, `SISMEMBER`, `SMEMBERS`, `SCARD`

### Connection Commands
- `PING`

### Generic Command Support
Any Redis command not explicitly listed above can still be executed through the generic endpoint. The system will attempt to execute it using Redis's native command interface.

## Response Format

All API responses follow this JSON structure:

```json
{
  "result": <redis_response>
}
```

Where `<redis_response>` is the actual Redis response converted to appropriate JSON types:
- Strings remain as strings
- Integers become JSON numbers
- Lists become JSON arrays
- Hashes become JSON objects
- Null values become JSON null

## Error Handling

Errors are returned with appropriate HTTP status codes and JSON error messages:

```json
{
  "error": "Error description here"
}
```

Common error status codes:
- `400 Bad Request` - Invalid command or parameters
- `401 Unauthorized` - Missing or invalid API key
- `404 Not Found` - Redis instance not found
- `500 Internal Server Error` - Redis connection or execution error

## Rate Limiting

API requests are subject to rate limiting based on your organization's plan. Exceeded rate limits will return `429 Too Many Requests`.

## Examples with Upstash Redis Client

This API is compatible with the Upstash Redis client. Here's how to configure it:

```javascript
import { Redis } from '@upstash/redis'

const redis = new Redis({
  url: 'http://localhost:8080/redis/your-instance-id',
  token: 'your-api-key'
})

// Use standard Redis operations
await redis.set('key', 'value')
const value = await redis.get('key')
await redis.hset('hash', { field: 'value' })
const hash = await redis.hgetall('hash')
```

## Security Considerations

1. **Always use HTTPS in production** to protect your API keys in transit
2. **Rotate API keys regularly** using the management API
3. **Use the minimum required scopes** when creating API keys
4. **Monitor API usage** through the management dashboard
5. **Store API keys securely** and never commit them to version control

## Development and Testing

For development environments, you can use the debug mode which provides additional logging and request information. The API includes comprehensive logging for debugging authentication and Redis operation issues.

To enable debug logging, set the `RUST_LOG` environment variable:
```bash
RUST_LOG=debug ./redisgate
```

This will show detailed information about API key validation, Redis connections, and command execution.