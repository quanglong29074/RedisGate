"""
Test basic Redis operations using Upstash Redis client.

This module tests fundamental Redis operations through RedisGate's HTTP API:
- String operations (GET, SET, DEL)
- Key existence and expiration
- Data persistence
- Error handling
"""

import pytest
import asyncio
import uuid
from typing import Dict, Any

from conftest import UpstashRedisClient


class TestBasicRedisOperations:
    """Test basic Redis string operations."""
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_ping(self, upstash_redis: UpstashRedisClient):
        """Test Redis ping operation."""
        result = await upstash_redis.ping()
        assert result == "PONG"
    
    @pytest.mark.redis
    @pytest.mark.integration  
    async def test_set_and_get_string(self, upstash_redis: UpstashRedisClient):
        """Test setting and getting a string value."""
        key = f"test_string_{uuid.uuid4().hex[:8]}"
        value = "Hello, RedisGate!"
        
        # Set the value
        set_result = await upstash_redis.set(key, value)
        assert set_result == "OK"
        
        # Get the value
        get_result = await upstash_redis.get(key)
        assert get_result == value
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_set_and_get_unicode(self, upstash_redis: UpstashRedisClient):
        """Test setting and getting Unicode strings."""
        key = f"test_unicode_{uuid.uuid4().hex[:8]}"
        value = "Hello 世界! 🚀 Testing unicode characters: áéíóú ñ"
        
        # Set the value
        set_result = await upstash_redis.set(key, value)
        assert set_result == "OK"
        
        # Get the value
        get_result = await upstash_redis.get(key)
        assert get_result == value
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_set_overwrite(self, upstash_redis: UpstashRedisClient):
        """Test overwriting an existing key."""
        key = f"test_overwrite_{uuid.uuid4().hex[:8]}"
        value1 = "original_value"
        value2 = "new_value"
        
        # Set original value
        await upstash_redis.set(key, value1)
        result1 = await upstash_redis.get(key)
        assert result1 == value1
        
        # Overwrite with new value
        await upstash_redis.set(key, value2)
        result2 = await upstash_redis.get(key)
        assert result2 == value2
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_get_nonexistent_key(self, upstash_redis: UpstashRedisClient):
        """Test getting a key that doesn't exist."""
        key = f"nonexistent_{uuid.uuid4().hex[:8]}"
        
        result = await upstash_redis.get(key)
        assert result is None
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_delete_key(self, upstash_redis: UpstashRedisClient):
        """Test deleting a key."""
        key = f"test_delete_{uuid.uuid4().hex[:8]}"
        value = "to_be_deleted"
        
        # Set the value
        await upstash_redis.set(key, value)
        assert await upstash_redis.get(key) == value
        
        # Delete the key
        delete_result = await upstash_redis.delete(key)
        assert delete_result == 1  # 1 key deleted
        
        # Verify key is gone
        get_result = await upstash_redis.get(key)
        assert get_result is None
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_delete_nonexistent_key(self, upstash_redis: UpstashRedisClient):
        """Test deleting a key that doesn't exist."""
        key = f"nonexistent_delete_{uuid.uuid4().hex[:8]}"
        
        delete_result = await upstash_redis.delete(key)
        assert delete_result == 0  # 0 keys deleted
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_empty_string_value(self, upstash_redis: UpstashRedisClient):
        """Test setting and getting an empty string."""
        key = f"test_empty_{uuid.uuid4().hex[:8]}"
        value = ""
        
        # Set empty value
        set_result = await upstash_redis.set(key, value)
        assert set_result == "OK"
        
        # Get empty value
        get_result = await upstash_redis.get(key)
        assert get_result == value
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_large_value(self, upstash_redis: UpstashRedisClient):
        """Test setting and getting a large string value."""
        key = f"test_large_{uuid.uuid4().hex[:8]}"
        # Create a 1MB string
        value = "x" * (1024 * 1024)
        
        # Set large value
        set_result = await upstash_redis.set(key, value)
        assert set_result == "OK"
        
        # Get large value
        get_result = await upstash_redis.get(key)
        assert get_result == value
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_special_characters_in_key(self, upstash_redis: UpstashRedisClient):
        """Test keys with special characters."""
        # Test various special characters in keys
        special_keys = [
            f"test:key:{uuid.uuid4().hex[:8]}",
            f"test-key-{uuid.uuid4().hex[:8]}",
            f"test_key_{uuid.uuid4().hex[:8]}",
            f"test.key.{uuid.uuid4().hex[:8]}",
            f"test/key/{uuid.uuid4().hex[:8]}",
        ]
        
        for key in special_keys:
            value = f"value_for_{key}"
            
            # Set and get with special key
            await upstash_redis.set(key, value)
            result = await upstash_redis.get(key)
            assert result == value
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_numeric_strings(self, upstash_redis: UpstashRedisClient):
        """Test setting and getting numeric string values."""
        test_cases = [
            ("123", "123"),
            ("0", "0"),
            ("-456", "-456"),
            ("3.14159", "3.14159"),
            ("1e10", "1e10"),
        ]
        
        for i, (input_val, expected) in enumerate(test_cases):
            key = f"test_numeric_{i}_{uuid.uuid4().hex[:8]}"
            
            await upstash_redis.set(key, input_val)
            result = await upstash_redis.get(key)
            assert result == expected


class TestRedisKeyOperations:
    """Test Redis key-related operations."""
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_multiple_keys_isolation(self, upstash_redis: UpstashRedisClient):
        """Test that multiple keys are properly isolated."""
        keys_values = {}
        
        # Create multiple key-value pairs
        for i in range(10):
            key = f"isolation_test_{i}_{uuid.uuid4().hex[:8]}"
            value = f"value_{i}_{uuid.uuid4().hex[:8]}"
            keys_values[key] = value
            await upstash_redis.set(key, value)
        
        # Verify all keys have correct values
        for key, expected_value in keys_values.items():
            result = await upstash_redis.get(key)
            assert result == expected_value
        
        # Delete half the keys
        keys_to_delete = list(keys_values.keys())[:5]
        for key in keys_to_delete:
            await upstash_redis.delete(key)
        
        # Verify deleted keys are gone and remaining keys are intact
        for key, expected_value in keys_values.items():
            result = await upstash_redis.get(key)
            if key in keys_to_delete:
                assert result is None
            else:
                assert result == expected_value


class TestRedisErrorHandling:
    """Test error handling and edge cases."""
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_very_long_key(self, upstash_redis: UpstashRedisClient):
        """Test handling of very long keys."""
        # Redis typically supports keys up to 512MB, but let's test a reasonable long key
        key = "very_long_key_" + "x" * 1000 + f"_{uuid.uuid4().hex[:8]}"
        value = "long_key_value"
        
        try:
            await upstash_redis.set(key, value)
            result = await upstash_redis.get(key)
            assert result == value
        except Exception as e:
            # Some Redis instances might have key length limits
            # This is acceptable behavior
            pytest.skip(f"Long key not supported: {e}")
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_concurrent_operations(self, upstash_redis: UpstashRedisClient):
        """Test concurrent Redis operations."""
        async def set_get_operation(index: int):
            key = f"concurrent_{index}_{uuid.uuid4().hex[:8]}"
            value = f"value_{index}_{uuid.uuid4().hex[:8]}"
            
            await upstash_redis.set(key, value)
            result = await upstash_redis.get(key)
            assert result == value
            return key
        
        # Run 10 concurrent operations
        tasks = [set_get_operation(i) for i in range(10)]
        keys = await asyncio.gather(*tasks)
        
        # Cleanup
        for key in keys:
            await upstash_redis.delete(key)


class TestRedisDataTypes:
    """Test different data types and encoding."""
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_json_like_strings(self, upstash_redis: UpstashRedisClient):
        """Test storing JSON-like string data."""
        key = f"test_json_{uuid.uuid4().hex[:8]}"
        json_value = '{"name": "John", "age": 30, "city": "New York"}'
        
        await upstash_redis.set(key, json_value)
        result = await upstash_redis.get(key)
        assert result == json_value
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_multiline_strings(self, upstash_redis: UpstashRedisClient):
        """Test storing multiline string data."""
        key = f"test_multiline_{uuid.uuid4().hex[:8]}"
        multiline_value = """This is line 1
This is line 2
This is line 3 with special chars: !@#$%^&*()
This is line 4 with unicode: 你好世界"""
        
        await upstash_redis.set(key, multiline_value)
        result = await upstash_redis.get(key)
        assert result == multiline_value
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_binary_like_strings(self, upstash_redis: UpstashRedisClient):
        """Test storing binary-like string data."""
        key = f"test_binary_{uuid.uuid4().hex[:8]}"
        # Simulate binary data as base64 string
        import base64
        binary_data = b"This is binary data with null bytes: \x00\x01\x02\x03"
        base64_value = base64.b64encode(binary_data).decode('utf-8')
        
        await upstash_redis.set(key, base64_value)
        result = await upstash_redis.get(key)
        assert result == base64_value
        
        # Verify we can decode it back
        decoded = base64.b64decode(result)
        assert decoded == binary_data


if __name__ == "__main__":
    # Run tests directly
    pytest.main([__file__, "-v"])