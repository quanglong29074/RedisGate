# RedisGate Development Test Suite

This directory contains a comprehensive test suite designed for development workflow testing of RedisGate. The test suite verifies all server APIs work correctly during local development.

## Overview

The development test suite is designed to be run by developers on their local machines after:
1. Running the setup script (`./setup-dev.sh`)
2. Building the project (`cargo build`)
3. Starting the server (`cargo run`)

The test suite verifies:
- **Public API endpoints** (health, version, stats) - no authentication required
- **Authentication endpoints** (register, login) - user management
- **Protected API endpoints** (organizations, api-keys, redis-instances) - JWT authentication required
- **Redis HTTP API endpoints** - API key authentication required

## Test Structure

```
test/development/
├── conftest.py                     # Test fixtures and configuration
├── pytest.ini                     # Pytest configuration
├── requirements.txt               # Python dependencies
├── run_tests.py                   # Main test runner script
├── test_public_endpoints.py       # Public API tests
├── test_auth_endpoints.py         # Authentication tests
├── test_protected_endpoints.py    # Protected API tests
├── test_redis_endpoints.py        # Redis HTTP API tests
└── README.md                      # This file
```

## Prerequisites

### System Requirements
- **Python**: 3.8 or higher
- **RedisGate server**: Running on localhost:8080 (default)
- **PostgreSQL**: For RedisGate's metadata storage (configured via setup script)

### RedisGate Server
The tests assume the RedisGate server is already running. Follow the development workflow:

```bash
# 1. One-time setup (install dependencies and start services)
./setup-dev.sh

# 2. Build the application
cargo build

# 3. Run the application (migrations run automatically)
cargo run
```

## Quick Start

### Automatic Setup (Recommended)
```bash
# Navigate to the development test directory
cd test/development

# Install dependencies and run all tests
python run_tests.py --install-deps

# Or if you prefer verbose output
python run_tests.py --install-deps -v
```

### Manual Setup
```bash
# Navigate to the development test directory
cd test/development

# Create virtual environment
python -m venv .venv

# Activate virtual environment
# On Linux/macOS:
source .venv/bin/activate
# On Windows:
.venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt

# Run tests
python run_tests.py
```

## Usage Examples

### Run All Tests
```bash
python run_tests.py
```

### Run Specific Test Categories
```bash
# Test only public endpoints
python run_tests.py -m public

# Test only authentication
python run_tests.py -m auth

# Test only protected endpoints
python run_tests.py -m protected

# Test only Redis HTTP API
python run_tests.py -m redis
```

### Run Specific Test Files
```bash
# Run only public endpoint tests
python run_tests.py test_public_endpoints.py

# Run multiple test files
python run_tests.py test_public_endpoints.py test_auth_endpoints.py
```

### Verbose Output and Reporting
```bash
# Verbose output
python run_tests.py -v

# Generate HTML report
python run_tests.py --report

# Generate both HTML and JSON reports
python run_tests.py --report --json-report
```

### Custom Server Configuration
```bash
# Test against custom host/port
python run_tests.py --host 192.168.1.100 --port 9000

# With verbose output
python run_tests.py --host localhost --port 8080 -v
```

## Test Categories

### Public Endpoints (`test_public_endpoints.py`)
Tests endpoints that don't require authentication:
- `GET /health` - Server health check
- `GET /version` - Server version information
- `GET /stats` - Database statistics

### Authentication (`test_auth_endpoints.py`)
Tests user authentication and registration:
- `POST /auth/register` - User registration
- `POST /auth/login` - User login
- JWT token validation
- Error handling for invalid credentials

### Protected Endpoints (`test_protected_endpoints.py`)
Tests endpoints that require JWT authentication:

**Organizations:**
- `POST /api/organizations` - Create organization
- `GET /api/organizations` - List organizations
- `GET /api/organizations/{id}` - Get organization
- `PUT /api/organizations/{id}` - Update organization
- `DELETE /api/organizations/{id}` - Delete organization

**API Keys:**
- `POST /api/organizations/{org_id}/api-keys` - Create API key
- `GET /api/organizations/{org_id}/api-keys` - List API keys
- `GET /api/organizations/{org_id}/api-keys/{key_id}` - Get API key
- `DELETE /api/organizations/{org_id}/api-keys/{key_id}` - Revoke API key

**Redis Instances:**
- `POST /api/organizations/{org_id}/redis-instances` - Create Redis instance
- `GET /api/organizations/{org_id}/redis-instances` - List Redis instances
- `GET /api/organizations/{org_id}/redis-instances/{id}` - Get Redis instance
- `PUT /api/organizations/{org_id}/redis-instances/{id}/status` - Update status
- `DELETE /api/organizations/{org_id}/redis-instances/{id}` - Delete Redis instance

### Redis HTTP API (`test_redis_endpoints.py`)
Tests Redis operations via HTTP API (requires API key authentication):
- `GET /redis/{instance_id}/ping` - PING command
- `GET /redis/{instance_id}/set/{key}/{value}` - SET command
- `GET /redis/{instance_id}/get/{key}` - GET command
- `GET /redis/{instance_id}/del/{key}` - DEL command
- `GET /redis/{instance_id}/incr/{key}` - INCR command
- `GET /redis/{instance_id}/hset/{key}/{field}/{value}` - HSET command
- `GET /redis/{instance_id}/hget/{key}/{field}` - HGET command
- `GET /redis/{instance_id}/lpush/{key}/{value}` - LPUSH command
- `GET /redis/{instance_id}/lpop/{key}` - LPOP command
- `POST /redis/{instance_id}` - Generic command execution

## Configuration

### Environment Variables
The tests use the following environment variables:

```bash
# Server configuration (set automatically by test runner)
REDISGATE_TEST_HOST=127.0.0.1
REDISGATE_TEST_PORT=8080

# Python environment
PYTHONPATH=./test/development
```

### Test Markers
Tests are categorized using pytest markers:

- `@pytest.mark.public` - Public API tests
- `@pytest.mark.auth` - Authentication tests
- `@pytest.mark.protected` - Protected API tests
- `@pytest.mark.redis` - Redis HTTP API tests
- `@pytest.mark.integration` - Full integration tests

## Development Workflow Integration

This test suite is designed to fit into the typical development workflow:

1. **Setup Development Environment:**
   ```bash
   ./setup-dev.sh
   ```

2. **Build Application:**
   ```bash
   cargo build
   ```

3. **Start Server:**
   ```bash
   cargo run
   ```

4. **Run Development Tests:**
   ```bash
   cd test/development
   python run_tests.py --install-deps
   ```

5. **Make Code Changes** (repeat steps 2-4 as needed)

## Troubleshooting

### Common Issues

**Server Not Running:**
```
✗ Server not available at http://127.0.0.1:8080
```
- Make sure you've run `cargo run` and the server started successfully
- Check that the server is running on the expected port (8080 by default)
- Verify no firewall is blocking the connection

**Dependency Issues:**
```
✗ Required dependencies not found
```
- Run with `--install-deps` flag to install dependencies automatically
- Or manually install: `pip install -r requirements.txt`

**Database Connection Errors:**
```
Database connection failed
```
- Ensure PostgreSQL is running (check with `./scripts/dev-services.sh status`)
- Verify the database configuration in `.env.development`
- Run database migrations: The server should do this automatically

**Test Failures:**
- Check server logs for errors
- Verify all external services (PostgreSQL) are running
- Run tests with `-v` flag for more detailed output

### Debug Mode
```bash
# Run with maximum verbosity
python run_tests.py -v -s

# Run single test file with debug output
python run_tests.py test_public_endpoints.py -v -s

# Generate detailed reports
python run_tests.py --report --json-report -v
```

## Integration with CI/CD

This test suite can be integrated into CI/CD pipelines. Example GitHub Actions workflow:

```yaml
name: Development Tests
on: [push, pull_request]

jobs:
  dev-tests:
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
    
    - name: Setup Development Environment
      run: ./setup-dev.sh
    
    - name: Build Application
      run: cargo build
    
    - name: Start Server
      run: cargo run &
      
    - name: Run Development Tests
      run: |
        cd test/development
        python run_tests.py --install-deps --report
        
    - name: Upload Test Results
      uses: actions/upload-artifact@v3
      if: always()
      with:
        name: test-results
        path: test/development/test_report.html
```

## Contributing

When adding new API endpoints to RedisGate:

1. **Add corresponding tests** to the appropriate test file
2. **Update test markers** if creating new endpoint categories
3. **Update this README** with new endpoint documentation
4. **Run the full test suite** to ensure no regressions

### Adding New Tests

1. **Choose the appropriate test file:**
   - Public endpoints → `test_public_endpoints.py`
   - Authentication → `test_auth_endpoints.py`
   - Protected endpoints → `test_protected_endpoints.py`
   - Redis operations → `test_redis_endpoints.py`

2. **Use appropriate pytest markers:**
   ```python
   @pytest.mark.public
   async def test_new_public_endpoint(self, api_client: ApiClient):
       # Test implementation
   ```

3. **Follow existing test patterns:**
   - Use fixtures for authentication and test data
   - Test both success and error cases
   - Use descriptive test names and docstrings

4. **Test the new functionality:**
   ```bash
   python run_tests.py test_new_file.py -v
   ```