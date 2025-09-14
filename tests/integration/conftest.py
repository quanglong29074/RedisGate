"""
Pytest configuration and fixtures for RedisGate integration tests.

This module provides:
- RedisGate server process management
- RedisGate Redis client setup  
- Test data generation and cleanup
- Authentication helpers
- Database setup and teardown
"""

import asyncio
import os
import signal
import socket
import subprocess
import tempfile
import time
import uuid
from pathlib import Path
from typing import AsyncGenerator, Generator, Optional, Dict, Any

import httpx
import pytest
import pytest_asyncio
from rich.console import Console
from rich.panel import Panel
import psutil

# Configure rich console for better test output
console = Console()

# Test configuration
TEST_HOST = "127.0.0.1"
TEST_PORT = 8080
TEST_DB_URL = "postgresql://redisgate_dev:redisgate_dev_password@localhost:5432/redisgate_dev"
TEST_JWT_SECRET = "test-jwt-secret-key-for-integration-tests"
SERVER_TIMEOUT = 60  # seconds to wait for server startup
CLIENT_TIMEOUT = 10   # seconds for client operations

class RedisGateServer:
    """Manages a RedisGate server process for testing."""
    
    def __init__(self, host: str = TEST_HOST, port: int = TEST_PORT, 
                 db_url: str = TEST_DB_URL, jwt_secret: str = TEST_JWT_SECRET):
        self.host = host
        self.port = port
        self.db_url = db_url
        self.jwt_secret = jwt_secret
        self.process: Optional[subprocess.Popen] = None
        self.temp_dir: Optional[Path] = None
        
    def setup_environment(self) -> Dict[str, str]:
        """Set up environment variables for the server."""
        env = os.environ.copy()
        env.update({
            "DATABASE_URL": self.db_url,
            "JWT_SECRET": self.jwt_secret,
            "RUST_LOG": "info",
            "REDISGATE_HOST": self.host,
            "REDISGATE_PORT": str(self.port),
        })
        return env
    
    def start(self, temp_dir: Optional[Path] = None) -> None:
        """Start the RedisGate server process."""
        if temp_dir:
            self.temp_dir = temp_dir
        else:
            self.temp_dir = Path(tempfile.mkdtemp())
        
        # Get the project root directory
        project_root = Path.cwd()
        if "tests" in str(project_root):
            # We're running from tests directory, go up to project root
            project_root = project_root.parent.parent
        
        # Build the server first
        console.print("[blue]Building RedisGate server...[/blue]")
        build_result = subprocess.run(
            ["cargo", "build"],
            cwd=project_root,
            capture_output=True,
            text=True
        )
        
        if build_result.returncode != 0:
            raise RuntimeError(f"Failed to build server: {build_result.stderr}")
        
        # Start the server process
        cmd = ["cargo", "run"]
        console.print(f"[blue]Starting RedisGate server: {' '.join(cmd)}[/blue]")
        
        env = self.setup_environment()
        
        self.process = subprocess.Popen(
            cmd,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            cwd=project_root,
            env=env
        )
        
        # Wait for server to be ready
        self._wait_for_server()
    
    async def start_async(self, temp_dir: Optional[Path] = None) -> None:
        """Start the RedisGate server process asynchronously."""
        # For now, use the sync version
        self.start(temp_dir)
    
    def _wait_for_server(self) -> None:
        """Wait for the server to be ready to accept connections."""
        console.print("[yellow]Waiting for server to start...[/yellow]")
        
        start_time = time.time()
        while time.time() - start_time < SERVER_TIMEOUT:
            # Check if process is still running
            if self.process.poll() is not None:
                # Process has exited, check for errors
                stdout, stderr = self.process.communicate()
                error_msg = f"Server process exited with code {self.process.returncode}"
                if stderr:
                    error_msg += f"\nStderr: {stderr.decode()}"
                if stdout:
                    error_msg += f"\nStdout: {stdout.decode()}"
                raise RuntimeError(error_msg)
            
            try:
                # Try to connect to the health endpoint
                response = httpx.get(f"http://{self.host}:{self.port}/health", timeout=1)
                if response.status_code == 200:
                    console.print("[green]Server is ready![/green]")
                    return
            except (httpx.ConnectError, httpx.TimeoutException):
                time.sleep(0.5)
                continue
        
        # If we get here, server didn't start in time
        if self.process.poll() is None:
            self.process.terminate()
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                self.process.kill()
                self.process.wait()
        
        raise TimeoutError(f"Server failed to start within {SERVER_TIMEOUT} seconds")
    
    def stop(self) -> None:
        """Stop the server process."""
        if self.process:
            console.print("[red]Stopping RedisGate server...[/red]")
            self.process.terminate()
            
            try:
                self.process.wait(timeout=5)
            except subprocess.TimeoutExpired:
                console.print("[red]Force killing server process...[/red]")
                self.process.kill()
                self.process.wait()
            
            self.process = None
    
    async def stop_async(self) -> None:
        """Stop the server process asynchronously."""
        self.stop()
    
    @property
    def base_url(self) -> str:
        """Get the base URL for the server."""
        return f"http://{self.host}:{self.port}"

class RedisGateClient:
    """HTTP client for interacting with RedisGate API."""
    
    def __init__(self, base_url: str, auth_token: Optional[str] = None):
        self.base_url = base_url.rstrip('/')
        self.auth_token = auth_token
        self.client = httpx.Client(timeout=CLIENT_TIMEOUT)
        
    def _get_headers(self) -> Dict[str, str]:
        """Get headers with authentication."""
        headers = {"Content-Type": "application/json"}
        if self.auth_token:
            headers["Authorization"] = f"Bearer {self.auth_token}"
        return headers
    
    async def register_user(self, email: str, username: str, password: str) -> Dict[str, Any]:
        """Register a new user."""
        data = {
            "email": email,
            "username": username,
            "password": password
        }
        response = self.client.post(f"{self.base_url}/auth/register", json=data)
        response.raise_for_status()
        return response.json()
    
    async def login(self, email: str, password: str) -> Dict[str, Any]:
        """Login and get auth token."""
        data = {"email": email, "password": password}
        response = self.client.post(f"{self.base_url}/auth/login", json=data)
        response.raise_for_status()
        result = response.json()
        # Extract token from the ApiResponse structure: result.data.token
        if "data" in result and result["data"] and "token" in result["data"]:
            self.auth_token = result["data"]["token"]
        return result
    
    async def create_organization(self, name: str, description: str = "") -> Dict[str, Any]:
        """Create a new organization."""
        # Generate a slug from the name (URL-friendly version)
        slug = name.lower().replace(" ", "-").replace("_", "-")
        # Remove any non-alphanumeric characters except hyphens
        import re
        slug = re.sub(r'[^a-z0-9-]', '', slug)
        
        data = {
            "name": name, 
            "slug": slug,
            "description": description
        }
        response = self.client.post(
            f"{self.base_url}/api/organizations", 
            json=data, 
            headers=self._get_headers()
        )
        response.raise_for_status()
        result = response.json()
        # Extract the organization data from the ApiResponse structure
        return result["data"] if "data" in result else result
    
    async def create_redis_instance(self, org_id: str, name: str, 
                                  memory_limit: int = 256) -> Dict[str, Any]:
        """Create a new Redis instance."""
        # Generate a slug from the name (URL-friendly version)
        slug = name.lower().replace(" ", "-").replace("_", "-")
        # Remove any non-alphanumeric characters except hyphens
        import re
        slug = re.sub(r'[^a-z0-9-]', '', slug)
        
        # Convert memory from MB to bytes (minimum 1MB = 1048576 bytes)
        memory_bytes = max(memory_limit * 1024 * 1024, 1048576)
        
        data = {
            "name": name,
            "slug": slug,
            "organization_id": org_id,
            "max_memory": memory_bytes
        }
        response = self.client.post(
            f"{self.base_url}/api/organizations/{org_id}/redis-instances", 
            json=data, 
            headers=self._get_headers()
        )
        response.raise_for_status()
        result = response.json()
        # Extract the data from the ApiResponse structure
        return result["data"] if "data" in result else result
    
    async def create_api_key(self, org_id: str, name: str, 
                           redis_instance_id: str) -> Dict[str, Any]:
        """Create a new API key for Redis instance."""
        # Standard scopes for Redis access
        scopes = ["redis:read", "redis:write", "redis:admin"]
        
        data = {
            "name": name,
            "organization_id": org_id,
            "scopes": scopes
        }
        response = self.client.post(
            f"{self.base_url}/api/organizations/{org_id}/api-keys", 
            json=data, 
            headers=self._get_headers()
        )
        response.raise_for_status()
        result = response.json()
        # Extract the data from the ApiResponse structure
        return result["data"] if "data" in result else result
    
    def close(self):
        """Close the HTTP client."""
        self.client.close()

class UpstashRedisClient:
    """RedisGate Redis client for testing Redis operations via HTTP API."""
    
    def __init__(self, redis_instance_url: str, api_key: str):
        self.redis_instance_url = redis_instance_url.rstrip('/')
        self.api_key = api_key
        self.client = httpx.AsyncClient(timeout=CLIENT_TIMEOUT)
        
        # Extract instance ID from URL if it's a full URL
        if 'redis/' in redis_instance_url:
            self.instance_id = redis_instance_url.split('redis/')[-1]
            self.base_url = redis_instance_url.split('/redis/')[0]
        else:
            # Assume it's just the instance ID
            self.instance_id = redis_instance_url
            self.base_url = "http://localhost:8080"
    
    def _get_headers(self) -> Dict[str, str]:
        """Get headers with API key authentication."""
        return {
            "Content-Type": "application/json",
            "Authorization": f"Bearer {self.api_key}"
        }
    
    def _get_params(self) -> Dict[str, str]:
        """Get query parameters with API key authentication."""
        return {"_token": self.api_key}
    
    async def set(self, key: str, value: str) -> Any:
        """Set a key-value pair."""
        url = f"{self.base_url}/redis/{self.instance_id}/set/{key}/{value}"
        response = await self.client.get(url, params=self._get_params())
        response.raise_for_status()
        result = response.json()
        # The server returns {"result": "OK"} for successful SET
        return result.get("result", "OK")
    
    async def get(self, key: str) -> Any:
        """Get a value by key."""
        url = f"{self.base_url}/redis/{self.instance_id}/get/{key}"
        response = await self.client.get(url, params=self._get_params())
        response.raise_for_status()
        result = response.json()
        # The server returns {"result": value} or {"result": null} for not found
        return result.get("result")
    
    async def delete(self, key: str) -> Any:
        """Delete a key."""
        url = f"{self.base_url}/redis/{self.instance_id}/del/{key}"
        response = await self.client.get(url, params=self._get_params())
        response.raise_for_status()
        result = response.json()
        # The server returns {"result": number_of_keys_deleted}
        return result.get("result", 0)
    
    async def ping(self) -> Any:
        """Ping the Redis instance."""
        url = f"{self.base_url}/redis/{self.instance_id}/ping"
        response = await self.client.get(url, params=self._get_params())
        response.raise_for_status()
        result = response.json()
        # The server returns {"result": "PONG"} for successful ping
        return result.get("result", "PONG")
    
    async def flushall(self) -> Any:
        """Flush all keys from the database."""
        # This would need to be implemented as a generic command
        url = f"{self.base_url}/redis/{self.instance_id}"
        payload = {"command": ["FLUSHALL"]}
        response = await self.client.post(url, json=payload, params=self._get_params())
        response.raise_for_status()
        result = response.json()
        # The server returns {"result": "OK"} for successful FLUSHALL
        return result.get("result", "OK")

# Fixtures

@pytest.fixture(scope="session")
def temp_test_dir() -> Generator[Path, None, None]:
    """Create a temporary directory for test data."""
    with tempfile.TemporaryDirectory() as temp_dir:
        yield Path(temp_dir)

@pytest.fixture(scope="session")
def server(temp_test_dir: Path) -> Generator[RedisGateServer, None, None]:
    """Provide a running RedisGate server for tests."""
    server = RedisGateServer()
    
    try:
        server.start(temp_test_dir)
        yield server
    finally:
        server.stop()

@pytest.fixture
def client(server: RedisGateServer) -> Generator[RedisGateClient, None, None]:
    """Provide an HTTP client for tests."""
    client = RedisGateClient(server.base_url)
    
    try:
        yield client
    finally:
        client.close()

@pytest.fixture
async def authenticated_client(client: RedisGateClient) -> AsyncGenerator[RedisGateClient, None]:
    """Provide an authenticated HTTP client."""
    # Create test user
    test_email = f"test-{uuid.uuid4()}@example.com"
    test_username = f"testuser-{uuid.uuid4().hex[:8]}"
    test_password = "testpassword123"
    
    await client.register_user(test_email, test_username, test_password)
    await client.login(test_email, test_password)
    
    yield client

@pytest.fixture
async def redis_setup(authenticated_client: RedisGateClient) -> AsyncGenerator[Dict[str, Any], None]:
    """Set up a Redis instance with API key for testing."""
    # Create organization
    org_name = f"test-org-{uuid.uuid4().hex[:8]}"
    org = await authenticated_client.create_organization(org_name, "Test organization")
    
    # Create Redis instance
    instance_name = f"test-redis-{uuid.uuid4().hex[:8]}"
    instance = await authenticated_client.create_redis_instance(
        org["id"], instance_name, 256
    )
    
    # Create API key
    api_key_name = f"test-key-{uuid.uuid4().hex[:8]}"
    api_key = await authenticated_client.create_api_key(
        org["id"], api_key_name, instance["id"]
    )
    
    # Wait a bit for the instance to be ready
    await asyncio.sleep(5)
    
    setup_data = {
        "organization": org,
        "instance": instance,
        "api_key": api_key,
        "redis_url": instance.get("endpoint_url", f"http://localhost:8080/redis/{instance['id']}"),
        "token": api_key["key"]
    }
    
    yield setup_data

@pytest.fixture
async def upstash_redis(redis_setup: Dict[str, Any]) -> AsyncGenerator[UpstashRedisClient, None]:
    """Provide a RedisGate Redis client for testing."""
    redis_client = UpstashRedisClient(
        redis_setup["redis_url"],
        redis_setup["token"]
    )
    
    try:
        yield redis_client
    finally:
        # Clean up by flushing all data
        try:
            await redis_client.flushall()
        except:
            pass  # Ignore cleanup errors

def pytest_configure(config):
    """Configure pytest with custom markers."""
    config.addinivalue_line(
        "markers", "slow: marks tests as slow (deselect with '-m \"not slow\"')"
    )
    config.addinivalue_line(
        "markers", "benchmark: marks tests as benchmark tests"
    )
    config.addinivalue_line(
        "markers", "integration: marks tests as integration tests"
    )
    config.addinivalue_line(
        "markers", "redis: marks tests that require Redis operations"
    )
    config.addinivalue_line(
        "markers", "auth: marks tests that require authentication"
    )
    config.addinivalue_line(
        "markers", "api: marks tests that exercise REST API"
    )

def pytest_collection_modifyitems(config, items):
    """Add markers to test items based on their names."""
    for item in items:
        if "benchmark" in item.name:
            item.add_marker(pytest.mark.benchmark)
        if "integration" in item.name:
            item.add_marker(pytest.mark.integration)
        if "redis" in item.name:
            item.add_marker(pytest.mark.redis)
        if "auth" in item.name:
            item.add_marker(pytest.mark.auth)
        if "api" in item.name:
            item.add_marker(pytest.mark.api)

@pytest.fixture(autouse=True)
def setup_logging():
    """Setup logging for tests."""
    # Set environment variables for logging
    os.environ["RUST_LOG"] = "info"
    yield

def generate_test_data(size: int = 100) -> Dict[str, str]:
    """Generate test key-value pairs."""
    import random
    import string
    
    data = {}
    for i in range(size):
        key = f"test_key_{i}"
        value = ''.join(random.choices(string.ascii_letters + string.digits, k=20))
        data[key] = value
    
    return data

def check_postgres_available() -> bool:
    """Check if PostgreSQL is available for testing."""
    try:
        import psycopg2
        conn = psycopg2.connect(TEST_DB_URL)
        conn.close()
        return True
    except:
        return False

def wait_for_postgres(timeout: int = 30) -> bool:
    """Wait for PostgreSQL to be available."""
    start_time = time.time()
    while time.time() - start_time < timeout:
        if check_postgres_available():
            return True
        time.sleep(1)
    return False