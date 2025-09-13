"""
Test API setup and authentication flow.

This module tests the complete API setup flow for RedisGate integration tests:
- User registration and authentication
- Organization creation
- Redis instance creation
- API key creation

This validates that the test infrastructure is working properly
and that all the management APIs are functional.
"""

import pytest
import uuid
from typing import Dict, Any

from conftest import RedisGateClient


class TestApiSetup:
    """Test the API setup flow for integration tests."""
    
    @pytest.mark.api
    @pytest.mark.integration
    async def test_complete_api_setup_flow(self, client: RedisGateClient):
        """Test the complete API setup flow used by integration tests."""
        # Create test user
        test_email = f"test-{uuid.uuid4()}@example.com"
        test_username = f"testuser-{uuid.uuid4().hex[:8]}"
        test_password = "testpassword123"
        
        # Step 1: Register user
        register_response = await client.register_user(test_email, test_username, test_password)
        assert "data" in register_response
        assert register_response["success"] is True
        
        # Step 2: Login
        login_response = await client.login(test_email, test_password)
        assert "data" in login_response
        assert "token" in login_response["data"]
        assert client.auth_token is not None
        
        # Step 3: Create organization
        org_name = f"test-org-{uuid.uuid4().hex[:8]}"
        org = await client.create_organization(org_name, "Test organization")
        assert "id" in org
        assert org["name"] == org_name
        
        # Step 4: Create Redis instance
        instance_name = f"test-redis-{uuid.uuid4().hex[:8]}"
        instance = await client.create_redis_instance(
            org["id"], instance_name, 256
        )
        assert "id" in instance
        assert instance["name"] == instance_name
        
        # Step 5: Create API key
        api_key_name = f"test-key-{uuid.uuid4().hex[:8]}"
        api_key_response = await client.create_api_key(
            org["id"], api_key_name, instance["id"]
        )
        # API key response has structure: {'api_key': {...}, 'key': '...'}
        assert "api_key" in api_key_response
        assert "key" in api_key_response
        assert api_key_response["api_key"]["name"] == api_key_name
        assert "id" in api_key_response["api_key"]
        
    @pytest.mark.api
    @pytest.mark.integration
    async def test_user_registration(self, client: RedisGateClient):
        """Test user registration API."""
        test_email = f"test-{uuid.uuid4()}@example.com"
        test_username = f"testuser-{uuid.uuid4().hex[:8]}"
        test_password = "testpassword123"
        
        response = await client.register_user(test_email, test_username, test_password)
        
        assert "data" in response
        assert response["success"] is True
        assert response["data"]["email"] == test_email
        assert response["data"]["username"] == test_username
        
    @pytest.mark.api
    @pytest.mark.integration
    async def test_user_login(self, client: RedisGateClient):
        """Test user login API."""
        test_email = f"test-{uuid.uuid4()}@example.com"
        test_username = f"testuser-{uuid.uuid4().hex[:8]}"
        test_password = "testpassword123"
        
        # Register user first
        await client.register_user(test_email, test_username, test_password)
        
        # Test login
        response = await client.login(test_email, test_password)
        
        assert "data" in response
        assert response["success"] is True
        assert "token" in response["data"]
        assert client.auth_token is not None
        
    @pytest.mark.api
    @pytest.mark.integration
    async def test_organization_creation(self, authenticated_client: RedisGateClient):
        """Test organization creation API."""
        org_name = f"test-org-{uuid.uuid4().hex[:8]}"
        description = "Test organization for API testing"
        
        response = await authenticated_client.create_organization(org_name, description)
        
        assert "id" in response
        assert response["name"] == org_name
        assert response["description"] == description
        assert "slug" in response
        
    @pytest.mark.api
    @pytest.mark.integration
    async def test_redis_instance_creation(self, authenticated_client: RedisGateClient):
        """Test Redis instance creation API."""
        # Create organization first
        org_name = f"test-org-{uuid.uuid4().hex[:8]}"
        org = await authenticated_client.create_organization(org_name, "Test organization")
        
        # Create Redis instance
        instance_name = f"test-redis-{uuid.uuid4().hex[:8]}"
        response = await authenticated_client.create_redis_instance(
            org["id"], instance_name, 256
        )
        
        assert "id" in response
        assert response["name"] == instance_name
        assert "slug" in response
        assert response["max_memory"] >= 256 * 1024 * 1024  # Converted to bytes
        
    @pytest.mark.api
    @pytest.mark.integration
    async def test_api_key_creation(self, authenticated_client: RedisGateClient):
        """Test API key creation API."""
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
        response = await authenticated_client.create_api_key(
            org["id"], api_key_name, instance["id"]
        )
        
        # API key response has structure: {'api_key': {...}, 'key': '...'}
        assert "api_key" in response
        assert "key" in response
        assert response["api_key"]["name"] == api_key_name
        assert "id" in response["api_key"]
        assert "scopes" in response["api_key"]
        assert len(response["api_key"]["scopes"]) > 0