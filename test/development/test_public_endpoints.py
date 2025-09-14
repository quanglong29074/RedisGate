"""
Test public API endpoints that don't require authentication.

This module tests:
- Health check endpoint
- Version endpoint  
- Database stats endpoint
"""

import pytest
from conftest import ApiClient


class TestPublicEndpoints:
    """Test public API endpoints that don't require authentication."""
    
    @pytest.mark.public
    async def test_health_check(self, api_client: ApiClient, wait_for_server):
        """Test the health check endpoint."""
        response = await api_client.get("/health")
        
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "healthy"
        assert "timestamp" in data
        assert "uptime" in data
    
    @pytest.mark.public
    async def test_version(self, api_client: ApiClient, wait_for_server):
        """Test the version endpoint."""
        response = await api_client.get("/version")
        
        assert response.status_code == 200
        data = response.json()
        assert "version" in data
        assert "build_time" in data
        assert "git_commit" in data
    
    @pytest.mark.public
    async def test_database_stats(self, api_client: ApiClient, wait_for_server):
        """Test the database statistics endpoint."""
        response = await api_client.get("/stats")
        
        assert response.status_code == 200
        data = response.json()
        assert "database" in data
        assert "connections" in data
        assert isinstance(data["connections"], dict)
        assert "pool_size" in data["connections"]
        assert "active_connections" in data["connections"]