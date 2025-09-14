"""
Pytest configuration and fixtures for RedisGate development test suite.

This module provides:
- Server URL configuration
- HTTP client setup
- Test data generation and cleanup
- Authentication helpers
"""

import asyncio
import os
import time
from pathlib import Path
from typing import AsyncGenerator, Dict, Any, Optional
from uuid import uuid4

import httpx
import pytest
import pytest_asyncio
from rich.console import Console

# Configure rich console for better test output
console = Console()

# Test configuration
TEST_HOST = os.getenv("REDISGATE_TEST_HOST", "127.0.0.1")
TEST_PORT = int(os.getenv("REDISGATE_TEST_PORT", "8080"))
BASE_URL = f"http://{TEST_HOST}:{TEST_PORT}"
CLIENT_TIMEOUT = 10   # seconds for client operations


class ApiClient:
    """Simple HTTP client wrapper for RedisGate API testing."""
    
    def __init__(self, base_url: str, timeout: int = CLIENT_TIMEOUT):
        self.base_url = base_url
        self.timeout = timeout
        self._client = None
        
    async def __aenter__(self):
        self._client = httpx.AsyncClient(
            base_url=self.base_url,
            timeout=self.timeout,
            follow_redirects=True
        )
        return self
        
    async def __aexit__(self, exc_type, exc_val, exc_tb):
        if self._client:
            await self._client.aclose()
    
    async def get(self, url: str, headers: Optional[Dict] = None, params: Optional[Dict] = None) -> httpx.Response:
        """Make GET request."""
        return await self._client.get(url, headers=headers, params=params)
    
    async def post(self, url: str, json: Optional[Dict] = None, headers: Optional[Dict] = None, params: Optional[Dict] = None) -> httpx.Response:
        """Make POST request."""
        return await self._client.post(url, json=json, headers=headers, params=params)
    
    async def put(self, url: str, json: Optional[Dict] = None, headers: Optional[Dict] = None) -> httpx.Response:
        """Make PUT request."""
        return await self._client.put(url, json=json, headers=headers)
    
    async def delete(self, url: str, headers: Optional[Dict] = None) -> httpx.Response:
        """Make DELETE request."""
        return await self._client.delete(url, headers=headers)


@pytest_asyncio.fixture
async def api_client() -> AsyncGenerator[ApiClient, None]:
    """Provide an HTTP client for API testing."""
    async with ApiClient(BASE_URL) as client:
        yield client


@pytest_asyncio.fixture
async def wait_for_server():
    """Wait for RedisGate server to be available."""
    max_attempts = 30
    for attempt in range(max_attempts):
        try:
            async with httpx.AsyncClient() as client:
                response = await client.get(f"{BASE_URL}/health", timeout=5.0)
                if response.status_code == 200:
                    console.print(f"[green]Server is ready at {BASE_URL}[/green]")
                    return
        except (httpx.ConnectError, httpx.TimeoutException):
            if attempt < max_attempts - 1:
                console.print(f"[yellow]Waiting for server... (attempt {attempt + 1}/{max_attempts})[/yellow]")
                await asyncio.sleep(2)
            else:
                raise ConnectionError(f"Server not available at {BASE_URL} after {max_attempts} attempts")


@pytest_asyncio.fixture
async def auth_user(api_client: ApiClient):
    """Create a test user and return authentication data."""
    # Generate unique test data
    username = f"testuser_{uuid4().hex[:8]}"
    email = f"{username}@example.com"
    password = "TestPassword123!"
    
    # Register user
    register_data = {
        "username": username,
        "email": email,
        "password": password
    }
    
    register_response = await api_client.post("/auth/register", json=register_data)
    assert register_response.status_code == 200, f"Registration failed: {register_response.text}"
    
    # Login to get JWT token
    login_data = {
        "email": email,
        "password": password
    }
    
    login_response = await api_client.post("/auth/login", json=login_data)
    assert login_response.status_code == 200, f"Login failed: {login_response.text}"
    
    login_result = login_response.json()
    
    return {
        "user_id": login_result["data"]["user"]["id"],
        "username": username,
        "email": email,
        "jwt_token": login_result["data"]["token"],
        "auth_headers": {"Authorization": f"Bearer {login_result['data']['token']}"}
    }


@pytest_asyncio.fixture  
async def test_organization(api_client: ApiClient, auth_user: Dict[str, Any]):
    """Create a test organization and return its data."""
    org_data = {
        "name": f"Test Organization {uuid4().hex[:8]}",
        "slug": f"test-org-{int(time.time() * 1000000)}",  # Use microsecond timestamp for uniqueness
        "description": "Test organization for development testing"
    }
    
    response = await api_client.post(
        "/api/organizations",
        json=org_data,
        headers=auth_user["auth_headers"]
    )
    assert response.status_code == 200, f"Organization creation failed: {response.text}"
    
    return response.json()["data"]


@pytest_asyncio.fixture
async def test_api_key(api_client: ApiClient, auth_user: Dict[str, Any], test_organization: Dict[str, Any]):
    """Create a test API key and return its data."""
    org_id = test_organization["id"]
    
    api_key_data = {
        "name": f"Test API Key {uuid4().hex[:8]}",
        "organization_id": org_id,
        "permissions": ["read", "write"],
        "expires_at": None  # No expiration for testing
    }
    
    response = await api_client.post(
        f"/api/organizations/{org_id}/api-keys",
        json=api_key_data,
        headers=auth_user["auth_headers"]
    )
    assert response.status_code == 200, f"API key creation failed: {response.text}"
    
    return response.json()["data"]


@pytest_asyncio.fixture
async def test_redis_instance(api_client: ApiClient, auth_user: Dict[str, Any], test_organization: Dict[str, Any]):
    """Create a test Redis instance and return its data."""
    org_id = test_organization["id"]
    
    instance_data = {
        "name": f"Test Redis Instance {uuid4().hex[:8]}",
        "redis_url": "redis://localhost:6379/0",  # Assuming local Redis for development
        "port": 6379,
        "database": 0,
        "max_connections": 10
    }
    
    response = await api_client.post(
        f"/api/organizations/{org_id}/redis-instances",
        json=instance_data,
        headers=auth_user["auth_headers"]
    )
    assert response.status_code == 200, f"Redis instance creation failed: {response.text}"
    
    return response.json()["data"]


def generate_test_key(prefix: str = "test") -> str:
    """Generate a unique test key."""
    return f"{prefix}_{uuid4().hex[:8]}"


def generate_test_value() -> str:
    """Generate a test value."""
    return f"value_{uuid4().hex[:8]}"