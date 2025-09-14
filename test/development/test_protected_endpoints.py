"""
Test protected API endpoints that require JWT authentication.

This module tests:
- Organization management endpoints
- API key management endpoints
- Redis instance management endpoints
"""

import pytest
import time
from uuid import uuid4
from typing import Dict, Any
from conftest import ApiClient, generate_test_key


class TestOrganizations:
    """Test organization management endpoints."""
    
    @pytest.mark.protected
    async def test_create_organization(self, api_client: ApiClient, auth_user: Dict[str, Any], wait_for_server):
        """Test creating an organization."""
        org_data = {
            "name": f"Test Organization {generate_test_key()}",
            "slug": f"test-org-{int(time.time() * 1000000)}",
            "description": "Test organization for development testing"
        }
        
        response = await api_client.post(
            "/api/organizations",
            json=org_data,
            headers=auth_user["auth_headers"]
        )
        
        assert response.status_code == 200
        data = response.json()
        assert data["success"] == True
        assert data["data"]["name"] == org_data["name"]
        assert data["data"]["description"] == org_data["description"]
        assert data["data"]["slug"] == org_data["slug"]
        assert "id" in data["data"]
        assert "owner_id" in data["data"]
        assert data["data"]["owner_id"] == auth_user["user_id"]
    
    @pytest.mark.protected
    async def test_list_organizations(self, api_client: ApiClient, auth_user: Dict[str, Any], test_organization: Dict[str, Any], wait_for_server):
        """Test listing organizations."""
        response = await api_client.get(
            "/api/organizations",
            headers=auth_user["auth_headers"]
        )
        
        assert response.status_code == 200
        data = response.json()
        assert data["success"] == True
        assert isinstance(data["data"]["items"], list)
        assert len(data["data"]["items"]) >= 1
        
        # Check if our test organization is in the list
        org_ids = [org["id"] for org in data["data"]["items"]]
        assert test_organization["id"] in org_ids
    
    @pytest.mark.protected
    async def test_get_organization(self, api_client: ApiClient, auth_user: Dict[str, Any], test_organization: Dict[str, Any], wait_for_server):
        """Test getting a specific organization."""
        org_id = test_organization["id"]
        
        response = await api_client.get(
            f"/api/organizations/{org_id}",
            headers=auth_user["auth_headers"]
        )
        
        assert response.status_code == 200
        data = response.json()
        assert data["success"] == True
        assert data["data"]["id"] == org_id
        assert data["data"]["name"] == test_organization["name"]
        assert data["data"]["description"] == test_organization["description"]
    
    @pytest.mark.protected
    async def test_update_organization(self, api_client: ApiClient, auth_user: Dict[str, Any], test_organization: Dict[str, Any], wait_for_server):
        """Test updating an organization."""
        org_id = test_organization["id"]
        
        update_data = {
            "name": f"Updated Organization {generate_test_key()}",
            "slug": f"updated-org-{int(time.time() * 1000000)}",
            "description": "Updated description"
        }
        
        response = await api_client.put(
            f"/api/organizations/{org_id}",
            json=update_data,
            headers=auth_user["auth_headers"]
        )
        
        assert response.status_code == 200
        data = response.json()
        assert data["success"] == True
        assert data["data"]["name"] == update_data["name"]
        assert data["data"]["description"] == update_data["description"]
    
    @pytest.mark.protected
    async def test_delete_organization(self, api_client: ApiClient, auth_user: Dict[str, Any], wait_for_server):
        """Test deleting an organization."""
        # Create a temporary organization for deletion
        org_data = {
            "name": f"Temp Organization {generate_test_key()}",
            "slug": f"temp-org-{int(time.time() * 1000000)}",
            "description": "Temporary organization for deletion test"
        }
        
        create_response = await api_client.post(
            "/api/organizations",
            json=org_data,
            headers=auth_user["auth_headers"]
        )
        assert create_response.status_code == 200
        temp_org = create_response.json()["data"]
        
        # Delete the organization
        delete_response = await api_client.delete(
            f"/api/organizations/{temp_org['id']}",
            headers=auth_user["auth_headers"]
        )
        
        assert delete_response.status_code == 200  # API returns 200, not 204
        
        # Verify it's deleted by trying to get it
        get_response = await api_client.get(
            f"/api/organizations/{temp_org['id']}",
            headers=auth_user["auth_headers"]
        )
        assert get_response.status_code == 404


class TestApiKeys:
    """Test API key management endpoints."""
    
    @pytest.mark.protected
    async def test_create_api_key(self, api_client: ApiClient, auth_user: Dict[str, Any], test_organization: Dict[str, Any], wait_for_server):
        """Test creating an API key."""
        org_id = test_organization["id"]
        
        api_key_data = {
            "name": f"Test API Key {generate_test_key()}",
            "permissions": ["read", "write"]
        }
        
        response = await api_client.post(
            f"/api/organizations/{org_id}/api-keys",
            json=api_key_data,
            headers=auth_user["auth_headers"]
        )
        
        assert response.status_code == 201
        data = response.json()
        assert data["name"] == api_key_data["name"]
        assert data["permissions"] == api_key_data["permissions"]
        assert "id" in data
        assert "key" in data
        assert "organization_id" in data
        assert data["organization_id"] == org_id
    
    @pytest.mark.protected
    async def test_list_api_keys(self, api_client: ApiClient, auth_user: Dict[str, Any], test_organization: Dict[str, Any], test_api_key: Dict[str, Any], wait_for_server):
        """Test listing API keys."""
        org_id = test_organization["id"]
        
        response = await api_client.get(
            f"/api/organizations/{org_id}/api-keys",
            headers=auth_user["auth_headers"]
        )
        
        assert response.status_code == 200
        data = response.json()
        assert isinstance(data, list)
        assert len(data) >= 1
        
        # Check if our test API key is in the list
        key_ids = [key["id"] for key in data]
        assert test_api_key["id"] in key_ids
    
    @pytest.mark.protected
    async def test_get_api_key(self, api_client: ApiClient, auth_user: Dict[str, Any], test_organization: Dict[str, Any], test_api_key: Dict[str, Any], wait_for_server):
        """Test getting a specific API key."""
        org_id = test_organization["id"]
        key_id = test_api_key["id"]
        
        response = await api_client.get(
            f"/api/organizations/{org_id}/api-keys/{key_id}",
            headers=auth_user["auth_headers"]
        )
        
        assert response.status_code == 200
        data = response.json()
        assert data["id"] == key_id
        assert data["name"] == test_api_key["name"]
        assert data["organization_id"] == org_id
    
    @pytest.mark.protected
    async def test_revoke_api_key(self, api_client: ApiClient, auth_user: Dict[str, Any], test_organization: Dict[str, Any], wait_for_server):
        """Test revoking an API key."""
        org_id = test_organization["id"]
        
        # Create a temporary API key for revocation
        api_key_data = {
            "name": f"Temp API Key {generate_test_key()}",
            "permissions": ["read"]
        }
        
        create_response = await api_client.post(
            f"/api/organizations/{org_id}/api-keys",
            json=api_key_data,
            headers=auth_user["auth_headers"]
        )
        assert create_response.status_code == 201
        temp_key = create_response.json()
        
        # Revoke the API key
        revoke_response = await api_client.delete(
            f"/api/organizations/{org_id}/api-keys/{temp_key['id']}",
            headers=auth_user["auth_headers"]
        )
        
        assert revoke_response.status_code == 204
        
        # Verify it's revoked by trying to get it
        get_response = await api_client.get(
            f"/api/organizations/{org_id}/api-keys/{temp_key['id']}",
            headers=auth_user["auth_headers"]
        )
        assert get_response.status_code == 404


class TestRedisInstances:
    """Test Redis instance management endpoints."""
    
    @pytest.mark.protected
    async def test_create_redis_instance(self, api_client: ApiClient, auth_user: Dict[str, Any], test_organization: Dict[str, Any], wait_for_server):
        """Test creating a Redis instance."""
        org_id = test_organization["id"]
        
        instance_data = {
            "name": f"Test Redis Instance {generate_test_key()}",
            "redis_url": "redis://localhost:6379/0",
            "port": 6379,
            "database": 0,
            "max_connections": 10
        }
        
        response = await api_client.post(
            f"/api/organizations/{org_id}/redis-instances",
            json=instance_data,
            headers=auth_user["auth_headers"]
        )
        
        assert response.status_code == 201
        data = response.json()
        assert data["name"] == instance_data["name"]
        assert data["redis_url"] == instance_data["redis_url"]
        assert data["port"] == instance_data["port"]
        assert data["database"] == instance_data["database"]
        assert "id" in data
        assert "organization_id" in data
        assert data["organization_id"] == org_id
    
    @pytest.mark.protected
    async def test_list_redis_instances(self, api_client: ApiClient, auth_user: Dict[str, Any], test_organization: Dict[str, Any], test_redis_instance: Dict[str, Any], wait_for_server):
        """Test listing Redis instances."""
        org_id = test_organization["id"]
        
        response = await api_client.get(
            f"/api/organizations/{org_id}/redis-instances",
            headers=auth_user["auth_headers"]
        )
        
        assert response.status_code == 200
        data = response.json()
        assert isinstance(data, list)
        assert len(data) >= 1
        
        # Check if our test Redis instance is in the list
        instance_ids = [instance["id"] for instance in data]
        assert test_redis_instance["id"] in instance_ids
    
    @pytest.mark.protected
    async def test_get_redis_instance(self, api_client: ApiClient, auth_user: Dict[str, Any], test_organization: Dict[str, Any], test_redis_instance: Dict[str, Any], wait_for_server):
        """Test getting a specific Redis instance."""
        org_id = test_organization["id"]
        instance_id = test_redis_instance["id"]
        
        response = await api_client.get(
            f"/api/organizations/{org_id}/redis-instances/{instance_id}",
            headers=auth_user["auth_headers"]
        )
        
        assert response.status_code == 200
        data = response.json()
        assert data["id"] == instance_id
        assert data["name"] == test_redis_instance["name"]
        assert data["organization_id"] == org_id
    
    @pytest.mark.protected
    async def test_update_redis_instance_status(self, api_client: ApiClient, auth_user: Dict[str, Any], test_organization: Dict[str, Any], test_redis_instance: Dict[str, Any], wait_for_server):
        """Test updating Redis instance status."""
        org_id = test_organization["id"]
        instance_id = test_redis_instance["id"]
        
        status_data = {
            "status": "active"
        }
        
        response = await api_client.put(
            f"/api/organizations/{org_id}/redis-instances/{instance_id}/status",
            json=status_data,
            headers=auth_user["auth_headers"]
        )
        
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == status_data["status"]
    
    @pytest.mark.protected
    async def test_delete_redis_instance(self, api_client: ApiClient, auth_user: Dict[str, Any], test_organization: Dict[str, Any], wait_for_server):
        """Test deleting a Redis instance."""
        org_id = test_organization["id"]
        
        # Create a temporary Redis instance for deletion
        instance_data = {
            "name": f"Temp Redis Instance {generate_test_key()}",
            "redis_url": "redis://localhost:6379/1",
            "port": 6379,
            "database": 1,
            "max_connections": 5
        }
        
        create_response = await api_client.post(
            f"/api/organizations/{org_id}/redis-instances",
            json=instance_data,
            headers=auth_user["auth_headers"]
        )
        assert create_response.status_code == 201
        temp_instance = create_response.json()
        
        # Delete the Redis instance
        delete_response = await api_client.delete(
            f"/api/organizations/{org_id}/redis-instances/{temp_instance['id']}",
            headers=auth_user["auth_headers"]
        )
        
        assert delete_response.status_code == 204
        
        # Verify it's deleted by trying to get it
        get_response = await api_client.get(
            f"/api/organizations/{org_id}/redis-instances/{temp_instance['id']}",
            headers=auth_user["auth_headers"]
        )
        assert get_response.status_code == 404


class TestUnauthorizedAccess:
    """Test that protected endpoints require authentication."""
    
    @pytest.mark.protected
    async def test_organizations_require_auth(self, api_client: ApiClient, wait_for_server):
        """Test that organization endpoints require authentication."""
        # Test without any headers
        response = await api_client.get("/api/organizations")
        assert response.status_code == 401
        
        response = await api_client.post("/api/organizations", json={"name": "test"})
        assert response.status_code == 401
    
    @pytest.mark.protected
    async def test_api_keys_require_auth(self, api_client: ApiClient, wait_for_server):
        """Test that API key endpoints require authentication."""
        fake_org_id = str(uuid4())
        
        response = await api_client.get(f"/api/organizations/{fake_org_id}/api-keys")
        assert response.status_code == 401
        
        response = await api_client.post(f"/api/organizations/{fake_org_id}/api-keys", json={"name": "test"})
        assert response.status_code == 401
    
    @pytest.mark.protected
    async def test_redis_instances_require_auth(self, api_client: ApiClient, wait_for_server):
        """Test that Redis instance endpoints require authentication."""
        fake_org_id = str(uuid4())
        
        response = await api_client.get(f"/api/organizations/{fake_org_id}/redis-instances")
        assert response.status_code == 401
        
        response = await api_client.post(f"/api/organizations/{fake_org_id}/redis-instances", json={"name": "test"})
        assert response.status_code == 401