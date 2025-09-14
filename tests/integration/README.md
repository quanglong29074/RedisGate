# RedisGate Integration Tests

This directory contains comprehensive integration tests for RedisGate using the Upstash Redis Python client to test the HTTP API endpoints.

## Overview

The integration tests verify RedisGate's functionality by:
- Setting up a complete RedisGate server environment
- Creating test users, organizations, and Redis instances
- Using the Upstash Redis client to perform HTTP-based Redis operations
- Testing both basic and advanced Redis functionality
- Performing performance benchmarks
- Validating error handling and edge cases

## Test Structure

```
tests/integration/
├── .gitignore              # Git ignore patterns for test artifacts
├── .python-version         # Python version specification
├── requirements.txt        # Python dependencies
├── pytest.ini            # Pytest configuration
├── conftest.py            # Test fixtures and configuration
├── run_tests.py           # Main test runner script
├── test_basic_redis_operations.py    # Basic Redis operations tests
├── test_advanced_redis_operations.py # Advanced Redis operations tests
├── test_complete_chain_integration.py # Complete chain integration tests
└── README.md              # This file
```

## Prerequisites

### System Requirements
- **Rust**: Latest stable version with Cargo
- **Python**: 3.8 or higher
- **PostgreSQL**: For RedisGate's metadata storage
- **Docker** (optional): For running PostgreSQL in a container

### RedisGate Server
The tests require a working RedisGate server installation. The test runner will:
1. Compile the RedisGate server using `cargo build`
2. Start the server with test configuration
3. Run tests against the running server
4. Clean up after tests complete

## Quick Start

### Automatic Setup (Recommended)
```bash
# Navigate to the integration tests directory
cd tests/integration

# Run tests with automatic dependency installation
python run_tests.py --mode basic --install-deps

# Run all tests with verbose output
python run_tests.py --mode all --install-deps --verbose --report
```

### Manual Setup
```bash
# 1. Create and activate a virtual environment
python3 -m venv .venv
source .venv/bin/activate  # On Windows: .venv\Scripts\activate

# 2. Install dependencies
pip install -r requirements.txt

# 3. Build RedisGate server (from project root)
cd ../..
cargo build
cd tests/integration

# 4. Run tests
python -m pytest -v
```

## Test Modes

The test runner supports several modes:

### Basic Mode (Default)
Tests fundamental Redis operations:
```bash
python run_tests.py --mode basic
```
- String operations (GET, SET, DELETE)
- Key management
- Unicode support
- Error handling

### Advanced Mode  
Tests complex Redis functionality:
```bash
python run_tests.py --mode advanced
```
- Pipeline-like operations (concurrent batching)
- Transaction simulation
- Data structure simulation (lists, hashes, sets using JSON)
- Advanced string operations

### Benchmark Mode
Performance and stress testing:
```bash
python run_tests.py --mode benchmark
```
- High-volume operations
- Concurrent access patterns
- Performance metrics
- Large data handling

### CI Mode
Optimized for continuous integration:
```bash
python run_tests.py --mode ci
```
- Excludes slow and benchmark tests
- Generates JUnit XML reports
- Optimized for automated testing

### All Mode
Runs comprehensive test suite:
```bash
python run_tests.py --mode all
```
- All tests except benchmarks
- Complete functionality validation

## Test Runner Options

```bash
python run_tests.py [options]

Options:
  --mode MODE           Test mode: basic, advanced, all, ci, benchmark
  --host HOST           Server host (default: 127.0.0.1)  
  --port PORT           Server port (default: 8080)
  --workers N           Number of parallel workers
  --verbose             Verbose output
  --report              Generate detailed report
  --install-deps        Install Python dependencies automatically
  --skip-server-build   Skip building the RedisGate server
  --timeout SECONDS     Test timeout in seconds (default: 300)
  --help                Show help message
```

## Test Organization

### Basic Redis Operations (`test_basic_redis_operations.py`)

**TestBasicRedisOperations**
- `test_ping()` - Redis connectivity
- `test_set_and_get_string()` - Basic string operations
- `test_set_and_get_unicode()` - Unicode support
- `test_set_overwrite()` - Key overwriting
- `test_get_nonexistent_key()` - Missing key handling
- `test_delete_key()` - Key deletion
- `test_empty_string_value()` - Empty value handling
- `test_large_value()` - Large data handling
- `test_special_characters_in_key()` - Special key formats
- `test_numeric_strings()` - Numeric data handling

**TestRedisKeyOperations**
- `test_multiple_keys_isolation()` - Key isolation verification

**TestRedisErrorHandling**
- `test_very_long_key()` - Edge case handling
- `test_concurrent_operations()` - Concurrent access

**TestRedisDataTypes**
- `test_json_like_strings()` - JSON data storage
- `test_multiline_strings()` - Multiline data
- `test_binary_like_strings()` - Binary data simulation

### Advanced Redis Operations (`test_advanced_redis_operations.py`)

**TestRedisPipelines**
- `test_pipeline_basic_operations()` - Batch operations
- `test_batch_operations_mixed()` - Mixed operation batches

**TestRedisTransactions**
- `test_atomic_operations_simulation()` - Atomic-like operations

**TestRedisAdvancedStringOperations**
- `test_string_operations_simulation()` - String manipulation

**TestRedisDataStructureSimulation**
- `test_list_simulation_with_json()` - List operations using JSON
- `test_hash_simulation_with_json()` - Hash operations using JSON
- `test_set_simulation_with_json()` - Set operations using JSON

**TestRedisBenchmarkOperations**
- `test_rapid_set_get_operations()` - Performance testing
- `test_large_batch_operations()` - Large-scale operations

### Complete Chain Integration Tests (`test_complete_chain_integration.py`)

**TestCompleteChainIntegration**
- `test_complete_end_to_end_chain()` - Complete workflow: register → organization → Redis instance → API key → Redis operations
- `test_chain_integration_multiple_resources()` - Multi-organization and multi-instance testing
- `test_chain_integration_performance()` - Performance testing of the complete chain workflow

These tests validate the entire user journey from account creation to Redis operations, providing comprehensive end-to-end testing of the RedisGate platform.

## Configuration

### Environment Variables
The tests use the following environment variables:

```bash
# Server configuration
REDISGATE_TEST_HOST=127.0.0.1
REDISGATE_TEST_PORT=8080

# Database configuration  
DATABASE_URL=postgresql://postgres:password@localhost:5432/redisgate_test
JWT_SECRET=test-jwt-secret-key-for-integration-tests

# Logging
RUST_LOG=info
```

### Pytest Configuration (`pytest.ini`)
- Test discovery patterns
- Markers for test categorization
- Timeout settings
- Output formatting
- Coverage configuration

## Test Data Management

### Fixtures
- **server**: Manages RedisGate server lifecycle
- **client**: HTTP client for API calls
- **authenticated_client**: Authenticated API client
- **redis_setup**: Complete Redis instance setup
- **upstash_redis**: Ready-to-use Upstash Redis client

### Data Generation
- Unique test keys using UUIDs
- Randomized test values
- Large dataset generation for stress testing
- Unicode and special character testing

### Cleanup
- Automatic cleanup after each test
- Database flushing between test runs
- Temporary file cleanup
- Server process termination

## Dependencies

### Core Testing Framework
- **pytest**: Test framework and runner
- **pytest-asyncio**: Async test support
- **pytest-benchmark**: Performance testing
- **pytest-xdist**: Parallel test execution
- **pytest-timeout**: Test timeout management

### Redis Client
- **upstash-redis**: Official Upstash Redis Python client
- **redis**: Alternative Redis client for comparison

### HTTP and Utilities
- **httpx**: Modern HTTP client for API calls
- **psutil**: Process management utilities
- **rich**: Enhanced console output (optional)
- **python-dotenv**: Environment variable management

### Test Utilities
- **Faker**: Test data generation
- **pydantic**: Data validation
- **toml**: Configuration file parsing

## Running Specific Tests

### By Test Name
```bash
# Run a specific test function
python -m pytest test_basic_redis_operations.py::TestBasicRedisOperations::test_ping -v

# Run all tests in a class
python -m pytest test_basic_redis_operations.py::TestBasicRedisOperations -v
```

### By Markers
```bash
# Run only Redis operation tests
python -m pytest -m redis -v

# Run integration tests excluding benchmarks
python -m pytest -m "integration and not benchmark" -v

# Run only benchmark tests
python -m pytest -m benchmark -v
```

### By Pattern
```bash
# Run tests matching a pattern
python -m pytest -k "test_set_and_get" -v

# Run tests excluding slow tests
python -m pytest -m "not slow" -v
```

## Performance Testing

### Benchmark Tests
The benchmark tests measure:
- Operations per second (OPS)
- Response latency
- Concurrent operation handling
- Large dataset processing
- Memory usage patterns

### Performance Metrics
Example output:
```
Performance: 45.23 SETs/sec, 67.89 GETs/sec
Successfully processed 1000 keys in batches of 50
```

### Tuning Parameters
Adjust these values in test files for different environments:
- `num_operations`: Number of operations for benchmarks
- `batch_size`: Batch size for large operations
- `concurrent_tasks`: Number of concurrent operations
- `timeout`: Operation timeout values

## Troubleshooting

### Common Issues

**Server Build Failures**
```bash
# Check Rust installation
cargo --version

# Clean and rebuild
cargo clean && cargo build
```

**Dependency Issues**
```bash
# Reinstall dependencies
rm -rf .venv
python -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt
```

**Database Connection Errors**
```bash
# Check PostgreSQL is running
# Update DATABASE_URL environment variable
# Ensure test database exists
```

**Port Conflicts**
```bash
# Change test port
python run_tests.py --port 8081
```

### Debug Mode
```bash
# Run with maximum verbosity
python run_tests.py --mode basic --verbose

# Run single test with debug output
python -m pytest test_basic_redis_operations.py::TestBasicRedisOperations::test_ping -v -s
```

### Log Analysis
Check logs in:
- Server stdout/stderr output
- `test_report.txt` (when using `--report`)
- Pytest output and error messages

## Contributing

### Adding New Tests
1. Follow existing test patterns
2. Use appropriate markers (`@pytest.mark.redis`, etc.)
3. Include proper cleanup in fixtures
4. Add comprehensive docstrings
5. Test both success and failure cases

### Test Naming Conventions
- Test files: `test_*.py`
- Test functions: `test_*()` 
- Test classes: `Test*`
- Descriptive names indicating what is being tested

### Markers
Use these markers to categorize tests:
- `@pytest.mark.redis` - Redis operation tests
- `@pytest.mark.integration` - Integration tests
- `@pytest.mark.benchmark` - Performance tests
- `@pytest.mark.slow` - Long-running tests
- `@pytest.mark.auth` - Authentication tests
- `@pytest.mark.api` - API endpoint tests

## Integration with CI/CD

### GitHub Actions Example
```yaml
name: Integration Tests
on: [push, pull_request]

jobs:
  integration-tests:
    runs-on: ubuntu-latest
    services:
      postgres:
        image: postgres:14
        env:
          POSTGRES_PASSWORD: password
        options: >-
          --health-cmd pg_isready
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
    - uses: actions/checkout@v3
    - uses: actions/setup-python@v4
      with:
        python-version: '3.11'
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
    
    - name: Run Integration Tests
      run: |
        cd tests/integration
        python run_tests.py --mode ci --install-deps
        
    - name: Upload Test Results
      uses: actions/upload-artifact@v3
      if: always()
      with:
        name: test-results
        path: tests/integration/test-results.xml
```

This comprehensive integration test suite ensures RedisGate's HTTP API works correctly with the Upstash Redis client and provides a solid foundation for continuous testing and development.