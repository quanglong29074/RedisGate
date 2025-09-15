"""
Test authentication endpoints.

This module tests:
- User registration
- User login
- JWT token validation
"""

import pytest
from uuid import uuid4
from conftest import ApiClient, generate_test_key


class TestAuthentication:
    """Test authentication related endpoints."""
    
    @pytest.mark.auth
    async def test_user_registration(self, api_client: ApiClient, wait_for_server):
        """Test user registration."""
        username = f"testuser_{generate_test_key()}"
        email = f"{username}@example.com"
        password = "TestPassword123!"
        
        register_data = {
            "username": username,
            "email": email,
            "password": password
        }
        
        response = await api_client.post("/auth/register", json=register_data)
        
        assert response.status_code == 200
        data = response.json()
        assert data["success"] == True
        assert data["data"]["username"] == username
        assert data["data"]["email"] == email
        assert "id" in data["data"]
    
    @pytest.mark.auth
    async def test_user_registration_duplicate_email(self, api_client: ApiClient, wait_for_server):
        """Test registration with duplicate email fails."""
        username = f"testuser_{generate_test_key()}"
        email = f"{username}@example.com"
        password = "TestPassword123!"
        
        register_data = {
            "username": username,
            "email": email,
            "password": password
        }
        
        # First registration should succeed
        response1 = await api_client.post("/auth/register", json=register_data)
        assert response1.status_code == 200
        
        # Second registration with same email should fail
        username2 = f"testuser2_{generate_test_key()}"
        register_data2 = {
            "username": username2,
            "email": email,  # Same email
            "password": password
        }
        
        response2 = await api_client.post("/auth/register", json=register_data2)
        assert response2.status_code == 409
        error_data = response2.json()
        assert error_data["success"] == False
    
    @pytest.mark.auth
    async def test_user_login(self, api_client: ApiClient, wait_for_server):
        """Test user login."""
        username = f"testuser_{generate_test_key()}"
        email = f"{username}@example.com"
        password = "TestPassword123!"
        
        # First register a user
        register_data = {
            "username": username,
            "email": email,
            "password": password
        }
        
        register_response = await api_client.post("/auth/register", json=register_data)
        assert register_response.status_code == 200
        
        # Then login
        login_data = {
            "email": email,
            "password": password
        }
        
        login_response = await api_client.post("/auth/login", json=login_data)
        
        assert login_response.status_code == 200
        data = login_response.json()
        assert data["success"] == True
        assert data["data"]["user"]["username"] == username
        assert data["data"]["user"]["email"] == email
        assert "id" in data["data"]["user"]
        assert "token" in data["data"]
        assert isinstance(data["data"]["token"], str)
        assert len(data["data"]["token"]) > 0
    
    @pytest.mark.auth
    async def test_user_login_invalid_credentials(self, api_client: ApiClient, wait_for_server):
        """Test login with invalid credentials fails."""
        login_data = {
            "email": "nonexistent@example.com",
            "password": "wrongpassword"
        }
        
        response = await api_client.post("/auth/login", json=login_data)
        
        assert response.status_code == 401
        error_data = response.json()
        assert error_data["success"] == False
    
    @pytest.mark.auth
    async def test_user_registration_invalid_data(self, api_client: ApiClient, wait_for_server):
        """Test registration with invalid data fails."""
        # Test missing required fields
        invalid_data = {
            "username": "testuser"
            # Missing email and password
        }
        
        response = await api_client.post("/auth/register", json=invalid_data)
        assert response.status_code == 422
        
        # Test invalid email format
        invalid_email_data = {
            "username": "testuser",
            "email": "invalid-email",
            "password": "TestPassword123!"
        }
        
        response = await api_client.post("/auth/register", json=invalid_email_data)
        assert response.status_code in [400, 422]  # Either is acceptable for validation errors
        
        # Test weak password
        weak_password_data = {
            "username": "testuser",
            "email": "test@example.com",
            "password": "123"  # Too weak
        }
        
        response = await api_client.post("/auth/register", json=weak_password_data)
        assert response.status_code in [400, 422]  # Either is acceptable for validation errors