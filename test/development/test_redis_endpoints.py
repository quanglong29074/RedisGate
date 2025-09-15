"""
Test Redis HTTP API endpoints that require API key authentication.

This module tests:
- Basic Redis operations (PING, GET, SET, DEL, INCR)
- Hash operations (HGET, HSET)
- List operations (LPUSH, LPOP)
- Generic command execution
- API key authentication
"""

import pytest
from typing import Dict, Any
from conftest import ApiClient, generate_test_key, generate_test_value


class TestRedisHttpApi:
    """Test Redis HTTP API endpoints."""
    
    @pytest.mark.redis
    async def test_redis_ping(self, api_client: ApiClient, test_redis_instance: Dict[str, Any], test_api_key: Dict[str, Any], wait_for_server):
        """Test Redis PING command."""
        instance_id = test_redis_instance["id"]
        api_key = test_api_key["key"]
        
        response = await api_client.get(
            f"/redis/{instance_id}/ping",
            headers={"Authorization": f"Bearer {api_key}"}
        )
        
        assert response.status_code == 200
        data = response.json()
        assert data["result"] == "PONG"
    
    @pytest.mark.redis
    async def test_redis_jwt_token_verification(self, api_client: ApiClient, test_redis_instance: Dict[str, Any], test_api_key: Dict[str, Any], wait_for_server):
        """Test that JWT tokens work for Redis API authentication."""
        instance_id = test_redis_instance["id"]
        jwt_token = test_api_key["key"]
        
        # Verify the token is a JWT format (has 3 parts separated by dots)
        assert isinstance(jwt_token, str)
        assert jwt_token.count('.') == 2, "API key should be a JWT token with 3 parts"
        assert len(jwt_token) > 100, "JWT token should be reasonably long"
        
        # Test that the JWT token works for Redis operations
        response = await api_client.get(
            f"/redis/{instance_id}/ping",
            headers={"Authorization": f"Bearer {jwt_token}"}
        )
        
        assert response.status_code == 200
        data = response.json()
        assert data["result"] == "PONG"
        
        # Test that JWT token works as query parameter too
        response_query = await api_client.get(
            f"/redis/{instance_id}/ping",
            params={"_token": jwt_token}
        )
        
        assert response_query.status_code == 200
        query_data = response_query.json()
        assert query_data["result"] == "PONG"
    
    @pytest.mark.redis
    async def test_redis_set_get(self, api_client: ApiClient, test_redis_instance: Dict[str, Any], test_api_key: Dict[str, Any], wait_for_server):
        """Test Redis SET and GET commands."""
        instance_id = test_redis_instance["id"]
        api_key = test_api_key["key"]
        key = generate_test_key("test_set_get")
        value = generate_test_value()
        
        # Test SET
        set_response = await api_client.get(
            f"/redis/{instance_id}/set/{key}/{value}",
            headers={"Authorization": f"Bearer {api_key}"}
        )
        
        assert set_response.status_code == 200
        set_data = set_response.json()
        assert set_data["result"] == "OK"
        
        # Test GET
        get_response = await api_client.get(
            f"/redis/{instance_id}/get/{key}",
            headers={"Authorization": f"Bearer {api_key}"}
        )
        
        assert get_response.status_code == 200
        get_data = get_response.json()
        assert get_data["result"] == value
    
    @pytest.mark.redis
    async def test_redis_del(self, api_client: ApiClient, test_redis_instance: Dict[str, Any], test_api_key: Dict[str, Any], wait_for_server):
        """Test Redis DEL command."""
        instance_id = test_redis_instance["id"]
        api_key = test_api_key["key"]
        key = generate_test_key("test_del")
        value = generate_test_value()
        
        # First set a value
        await api_client.get(
            f"/redis/{instance_id}/set/{key}/{value}",
            headers={"Authorization": f"Bearer {api_key}"}
        )
        
        # Then delete it
        del_response = await api_client.get(
            f"/redis/{instance_id}/del/{key}",
            headers={"Authorization": f"Bearer {api_key}"}
        )
        
        assert del_response.status_code == 200
        del_data = del_response.json()
        assert del_data["result"] == 1  # Number of keys deleted
        
        # Verify it's deleted
        get_response = await api_client.get(
            f"/redis/{instance_id}/get/{key}",
            headers={"Authorization": f"Bearer {api_key}"}
        )
        
        assert get_response.status_code == 200
        get_data = get_response.json()
        assert get_data["result"] is None
    
    @pytest.mark.redis
    async def test_redis_incr(self, api_client: ApiClient, test_redis_instance: Dict[str, Any], test_api_key: Dict[str, Any], wait_for_server):
        """Test Redis INCR command."""
        instance_id = test_redis_instance["id"]
        api_key = test_api_key["key"]
        key = generate_test_key("test_incr")
        
        # Test INCR on non-existent key (should start from 0)
        incr_response = await api_client.get(
            f"/redis/{instance_id}/incr/{key}",
            headers={"Authorization": f"Bearer {api_key}"}
        )
        
        assert incr_response.status_code == 200
        incr_data = incr_response.json()
        assert incr_data["result"] == 1
        
        # Test INCR again
        incr_response2 = await api_client.get(
            f"/redis/{instance_id}/incr/{key}",
            headers={"Authorization": f"Bearer {api_key}"}
        )
        
        assert incr_response2.status_code == 200
        incr_data2 = incr_response2.json()
        assert incr_data2["result"] == 2
    
    @pytest.mark.redis
    async def test_redis_hset_hget(self, api_client: ApiClient, test_redis_instance: Dict[str, Any], test_api_key: Dict[str, Any], wait_for_server):
        """Test Redis HSET and HGET commands."""
        instance_id = test_redis_instance["id"]
        api_key = test_api_key["key"]
        key = generate_test_key("test_hash")
        field = "test_field"
        value = generate_test_value()
        
        # Test HSET
        hset_response = await api_client.get(
            f"/redis/{instance_id}/hset/{key}/{field}/{value}",
            headers={"Authorization": f"Bearer {api_key}"}
        )
        
        assert hset_response.status_code == 200
        hset_data = hset_response.json()
        assert hset_data["result"] == 1  # Number of fields added
        
        # Test HGET
        hget_response = await api_client.get(
            f"/redis/{instance_id}/hget/{key}/{field}",
            headers={"Authorization": f"Bearer {api_key}"}
        )
        
        assert hget_response.status_code == 200
        hget_data = hget_response.json()
        assert hget_data["result"] == value
    
    @pytest.mark.redis
    async def test_redis_lpush_lpop(self, api_client: ApiClient, test_redis_instance: Dict[str, Any], test_api_key: Dict[str, Any], wait_for_server):
        """Test Redis LPUSH and LPOP commands."""
        instance_id = test_redis_instance["id"]
        api_key = test_api_key["key"]
        key = generate_test_key("test_list")
        value = generate_test_value()
        
        # Test LPUSH
        lpush_response = await api_client.get(
            f"/redis/{instance_id}/lpush/{key}/{value}",
            headers={"Authorization": f"Bearer {api_key}"}
        )
        
        assert lpush_response.status_code == 200
        lpush_data = lpush_response.json()
        assert lpush_data["result"] == 1  # Length of list after push
        
        # Test LPOP
        lpop_response = await api_client.get(
            f"/redis/{instance_id}/lpop/{key}",
            headers={"Authorization": f"Bearer {api_key}"}
        )
        
        assert lpop_response.status_code == 200
        lpop_data = lpop_response.json()
        assert lpop_data["result"] == value
    
    @pytest.mark.redis
    async def test_redis_generic_command(self, api_client: ApiClient, test_redis_instance: Dict[str, Any], test_api_key: Dict[str, Any], wait_for_server):
        """Test generic Redis command execution via POST."""
        instance_id = test_redis_instance["id"]
        api_key = test_api_key["key"]
        key = generate_test_key("test_generic")
        value = generate_test_value()
        
        # Test generic SET command
        set_command = ["SET", key, value]
        
        set_response = await api_client.post(
            f"/redis/{instance_id}",
            json=set_command,
            headers={"Authorization": f"Bearer {api_key}"}
        )
        
        assert set_response.status_code == 200
        set_data = set_response.json()
        assert set_data["result"] == "OK"
        
        # Test generic GET command
        get_command = ["GET", key]
        
        get_response = await api_client.post(
            f"/redis/{instance_id}",
            json=get_command,
            headers={"Authorization": f"Bearer {api_key}"}
        )
        
        assert get_response.status_code == 200
        get_data = get_response.json()
        assert get_data["result"] == value
    
    @pytest.mark.redis
    async def test_redis_api_key_query_param(self, api_client: ApiClient, test_redis_instance: Dict[str, Any], test_api_key: Dict[str, Any], wait_for_server):
        """Test Redis API with API key as query parameter."""
        instance_id = test_redis_instance["id"]
        api_key = test_api_key["key"]
        
        response = await api_client.get(
            f"/redis/{instance_id}/ping",
            params={"_token": api_key}
        )
        
        assert response.status_code == 200
        data = response.json()
        assert data["result"] == "PONG"
    
    @pytest.mark.redis
    async def test_redis_unauthorized_access(self, api_client: ApiClient, test_redis_instance: Dict[str, Any], wait_for_server):
        """Test that Redis endpoints require API key authentication."""
        instance_id = test_redis_instance["id"]
        
        # Test without API key
        response = await api_client.get(f"/redis/{instance_id}/ping")
        assert response.status_code == 401
        
        # Test with invalid API key
        response = await api_client.get(
            f"/redis/{instance_id}/ping",
            headers={"Authorization": "Bearer invalid_key"}
        )
        assert response.status_code == 401
    
    @pytest.mark.redis
    async def test_redis_nonexistent_instance(self, api_client: ApiClient, test_api_key: Dict[str, Any], wait_for_server):
        """Test Redis operations with non-existent instance ID."""
        fake_instance_id = "00000000-0000-0000-0000-000000000000"
        api_key = test_api_key["key"]
        
        response = await api_client.get(
            f"/redis/{fake_instance_id}/ping",
            headers={"Authorization": f"Bearer {api_key}"}
        )
        
        assert response.status_code == 404
    
    @pytest.mark.redis
    async def test_redis_debug_endpoint(self, api_client: ApiClient, test_redis_instance: Dict[str, Any], test_api_key: Dict[str, Any], wait_for_server):
        """Test Redis debug endpoint."""
        instance_id = test_redis_instance["id"]
        api_key = test_api_key["key"]
        
        response = await api_client.get(
            f"/redis/{instance_id}/debug/test",
            headers={"Authorization": f"Bearer {api_key}"}
        )
        
        # Debug endpoint should return information about the request
        assert response.status_code == 200
        data = response.json()
        assert "method" in data
        assert "path" in data
        assert "instance_id" in data
        assert data["instance_id"] == instance_id