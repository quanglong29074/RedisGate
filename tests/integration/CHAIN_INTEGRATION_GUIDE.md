# Chain Integration Test Usage Guide

This guide explains how to run and interpret the comprehensive chain integration tests for RedisGate.

## Overview

The chain integration tests validate the complete end-to-end workflow:
1. **Register Account** - Create a new user account
2. **Create Organization** - Set up an organization for the user
3. **Create Redis Instance** - Deploy a Redis instance via Kubernetes
4. **Create API Key** - Generate credentials for Redis access
5. **Test Redis Operations** - Perform SET/GET operations

## Running the Tests

### Prerequisites

1. **PostgreSQL** must be running:
   ```bash
   ./scripts/dev-services.sh start
   ```

2. **Environment Variables**:
   ```bash
   export DATABASE_URL="postgresql://redisgate_dev:redisgate_dev_password@localhost:5432/redisgate_dev"
   export JWT_SECRET="test-jwt-secret-key-for-integration-tests"
   ```

3. **Python Environment**:
   ```bash
   cd tests/integration
   python -m venv .venv
   source .venv/bin/activate
   pip install -r requirements.txt
   ```

### Run All Chain Tests

```bash
# All chain integration tests
python -m pytest test_complete_chain_integration.py -v

# Single complete chain test
python -m pytest test_complete_chain_integration.py::TestCompleteChainIntegration::test_complete_end_to_end_chain -v -s

# Multi-resource test
python -m pytest test_complete_chain_integration.py::TestCompleteChainIntegration::test_chain_integration_multiple_resources -v -s

# Performance test
python -m pytest test_complete_chain_integration.py::TestCompleteChainIntegration::test_chain_integration_performance -v -s
```

### Using the Test Runner

```bash
# Use the provided test runner for easy execution
python run_tests.py --mode basic --verbose
```

## Understanding Test Results

### Expected Behavior (Without Kubernetes)

```
🏁 CHAIN INTEGRATION TEST SUMMARY for [test-id]
============================================================
✅ Step 1: User registration - SUCCESS
✅ Step 2: User authentication - SUCCESS  
✅ Step 3: Organization creation - SUCCESS
✅ Step 4: Redis instance creation - SUCCESS
✅ Step 5: API key creation - SUCCESS
✅ Step 6: Redis operations (SET/GET/DELETE) - SUCCESS

📊 CHAIN TEST RESULTS:
   Management API Flow: ✅ COMPLETE
   Redis Operations: ✅ WORKING
```

**Note**: Redis operations now work even without Kubernetes by connecting to the local Redis instance provided by `./scripts/dev-services.sh start`. The server creates database records for Redis instances (status: "simulation") and the Redis HTTP API connects to the local Redis service.

### Expected Behavior (With Kubernetes)

In a complete environment with Kubernetes, the main difference is that Redis instances are actually deployed to the Kubernetes cluster with status "pending" or "running", rather than status "simulation". The test behavior and results are the same:

```
🏁 CHAIN INTEGRATION TEST SUMMARY for [test-id]
============================================================
✅ Step 1: User registration - SUCCESS
✅ Step 2: User authentication - SUCCESS  
✅ Step 3: Organization creation - SUCCESS
✅ Step 4: Redis instance creation - SUCCESS
✅ Step 5: API key creation - SUCCESS
✅ Step 6: Redis operations (SET/GET/DELETE) - SUCCESS

📊 CHAIN TEST RESULTS:
   Management API Flow: ✅ COMPLETE
   Redis Operations: ✅ WORKING
```

### Performance Metrics

The performance test provides timing information:

```
📊 PERFORMANCE RESULTS:
   User Registration: 0.938s
   User Login: 0.912s
   Organization Creation: 0.014s
   Redis Instance Creation: 1.822s
   Total Management API Time: 3.685s
```

## Test Scenarios

### 1. Complete End-to-End Chain

**Purpose**: Validates the entire user workflow from registration to Redis operations.

**What it tests**:
- User account creation and authentication
- Organization management
- Redis instance provisioning (with Kubernetes detection)
- API key management
- Redis operations (SET/GET/DELETE)

**Key validation points**:
- All management API endpoints work correctly
- Data flow between components is correct
- Environment-aware behavior based on Kubernetes availability
- Proper error handling for infrastructure deployment failures

**Kubernetes Detection**:
The test automatically detects if Kubernetes is available by:
1. Checking the `KUBERNETES_AVAILABLE` environment variable (set in CI)
2. Attempting to connect to a Kubernetes cluster via `kubectl`

**Behavior**:
- **With Kubernetes**: Expects Redis instance creation and operations to succeed
- **Without Kubernetes**: Gracefully handles deployment failures and tests management API only
- Resource isolation and security

### 2. Multiple Resources

**Purpose**: Tests system behavior with multiple organizations and Redis instances.

**What it tests**:
- Creating multiple organizations under one user
- Creating multiple Redis instances across organizations
- Resource isolation between organizations
- System scalability with multiple resources

**Key validation points**:
- Unique resource IDs are generated
- Proper organization-instance associations
- No resource leakage between organizations

### 3. Performance Testing

**Purpose**: Measures the performance of the complete workflow.

**What it tests**:
- Time taken for each step in the chain
- Overall system performance under load
- Performance thresholds and SLA validation

**Key validation points**:
- User registration < 5 seconds
- User login < 3 seconds
- Organization creation < 3 seconds
- Redis instance creation attempt < 10 seconds
- Total workflow < 20 seconds

## Interpreting Failures

### Redis Instance Creation Failures

**Error**: `Server error '500 Internal Server Error'`

**Cause**: Kubernetes cluster not available or configured

**Resolution**: 
- In development: Expected behavior, test continues with simulation
- In production: Ensure Kubernetes cluster is properly configured

### API Key Creation Failures

**Error**: Authentication or authorization errors

**Cause**: 
- Invalid JWT token
- Missing organization membership
- Database connectivity issues

**Resolution**: Check authentication flow and database connectivity

### Redis Operations Failures

**Error**: Connection timeout or authentication errors

**Cause**:
- Redis instance not deployed
- Invalid API key
- Network connectivity issues

**Resolution**: Ensure Redis instance is deployed and accessible

## Debugging Tips

### Enable Verbose Logging

```bash
# Run with detailed output
python -m pytest test_complete_chain_integration.py -v -s --tb=long

# Enable HTTP request logging
export HTTPX_LOG_LEVEL=DEBUG
```

### Check Server Logs

The test automatically starts a RedisGate server instance. Monitor its output for detailed error information.

### Database State

```bash
# Connect to test database to check state
psql "postgresql://redisgate_dev:redisgate_dev_password@localhost:5432/redisgate_dev"

# Check created resources
SELECT * FROM users;
SELECT * FROM organizations;
SELECT * FROM redis_instances;
SELECT * FROM api_keys;
```

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Chain Integration Tests
on: [push, pull_request]

jobs:
  chain-integration:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:15-alpine
        env:
          POSTGRES_PASSWORD: redisgate_dev_password
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
    - uses: actions/checkout@v3
    - uses: actions/setup-python@v4
    
    # Set up Kubernetes for full chain testing
    - name: Set up Kubernetes (minikube)
      run: |
        curl -LO https://storage.googleapis.com/minikube/releases/latest/minikube-linux-amd64
        sudo install minikube-linux-amd64 /usr/local/bin/minikube
        minikube start --driver=docker
        minikube addons enable ingress
        echo "KUBERNETES_AVAILABLE=true" >> $GITHUB_ENV
    
    - name: Run Chain Integration Tests
      run: |
        cd tests/integration
        pip install -r requirements.txt
        python -m pytest test_complete_chain_integration.py -v
```

**Note:** With Kubernetes setup, the chain integration tests will validate the complete end-to-end workflow including Redis instance deployment and operations.

### Local Development

```bash
# Quick validation
make test-integration-chain

# Full test suite
make test-integration
```

## Customization

### Extending the Chain Test

To add new steps to the chain:

1. Add the step to `test_complete_end_to_end_chain()`
2. Update the validation logic
3. Add corresponding error handling
4. Update the summary output

### Adding New Chain Variants

Create new test methods following the pattern:

```python
@pytest.mark.integration
@pytest.mark.api
async def test_chain_integration_custom_scenario(self, client: RedisGateClient):
    """Test custom chain integration scenario."""
    # Your custom test logic here
```

## Troubleshooting

### Common Issues

1. **PostgreSQL not running**: Start with `./scripts/dev-services.sh start`
2. **Database migrations not applied**: Run `sqlx migrate run`
3. **Python dependencies missing**: Install with `pip install -r requirements.txt`
4. **Port conflicts**: Change test port with `--port` argument

### Getting Help

1. Check the test output for detailed error messages
2. Review the server logs for backend issues
3. Validate database state for data consistency
4. Check network connectivity for Redis operations

The chain integration tests provide comprehensive validation of the RedisGate platform and serve as both functional tests and living documentation of the expected system behavior.