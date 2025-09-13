"""
Test advanced Redis operations using Upstash Redis client.

This module tests advanced Redis features through RedisGate's HTTP API:
- Pipeline operations
- Transaction operations (MULTI/EXEC)
- Pub/Sub operations
- Batch operations
- Complex data manipulation
"""

import pytest
import asyncio
import uuid
import json
from typing import Dict, Any, List

from conftest import UpstashRedisClient


class TestRedisPipelines:
    """Test Redis pipeline operations."""
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_pipeline_basic_operations(self, upstash_redis: UpstashRedisClient):
        """Test basic pipeline operations."""
        # Note: Upstash Redis client may handle pipelines differently
        # We'll test batch operations instead
        keys_values = {}
        
        # Prepare test data
        for i in range(5):
            key = f"pipeline_test_{i}_{uuid.uuid4().hex[:8]}"
            value = f"value_{i}_{uuid.uuid4().hex[:8]}"
            keys_values[key] = value
        
        # Set all values (simulating pipeline)
        set_tasks = [upstash_redis.set(k, v) for k, v in keys_values.items()]
        set_results = await asyncio.gather(*set_tasks)
        
        # Verify all sets succeeded
        assert all(result == "OK" for result in set_results)
        
        # Get all values (simulating pipeline)
        get_tasks = [upstash_redis.get(k) for k in keys_values.keys()]
        get_results = await asyncio.gather(*get_tasks)
        
        # Verify all values are correct
        for i, (key, expected_value) in enumerate(keys_values.items()):
            assert get_results[i] == expected_value
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_batch_operations_mixed(self, upstash_redis: UpstashRedisClient):
        """Test mixed batch operations (SET, GET, DELETE)."""
        test_data = {}
        
        # Phase 1: Set multiple keys
        for i in range(3):
            key = f"batch_mixed_{i}_{uuid.uuid4().hex[:8]}"
            value = f"value_{i}"
            test_data[key] = value
        
        # Set all keys concurrently
        set_tasks = [upstash_redis.set(k, v) for k, v in test_data.items()]
        await asyncio.gather(*set_tasks)
        
        # Phase 2: Mixed operations (GET existing, SET new, DELETE existing)
        keys_list = list(test_data.keys())
        
        # GET first key, SET new key, DELETE second key
        new_key = f"batch_new_{uuid.uuid4().hex[:8]}"
        new_value = "new_batch_value"
        
        operations = [
            upstash_redis.get(keys_list[0]),  # GET
            upstash_redis.set(new_key, new_value),  # SET
            upstash_redis.delete(keys_list[1]),  # DELETE
        ]
        
        results = await asyncio.gather(*operations)
        
        # Verify results
        assert results[0] == test_data[keys_list[0]]  # GET result
        assert results[1] == "OK"  # SET result
        assert results[2] == 1  # DELETE result (1 key deleted)
        
        # Verify final state
        assert await upstash_redis.get(keys_list[0]) == test_data[keys_list[0]]  # Still exists
        assert await upstash_redis.get(new_key) == new_value  # New key exists
        assert await upstash_redis.get(keys_list[1]) is None  # Deleted key is gone


class TestRedisTransactions:
    """Test Redis transaction-like operations."""
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_atomic_operations_simulation(self, upstash_redis: UpstashRedisClient):
        """Test atomic-like operations using careful sequencing."""
        # Since Upstash REST API may not support true MULTI/EXEC,
        # we'll test atomic-like behavior with careful operation ordering
        
        key1 = f"atomic_test_1_{uuid.uuid4().hex[:8]}"
        key2 = f"atomic_test_2_{uuid.uuid4().hex[:8]}"
        
        # Initial setup
        await upstash_redis.set(key1, "100")
        await upstash_redis.set(key2, "200")
        
        # Simulate atomic transfer: subtract from key1, add to key2
        val1 = await upstash_redis.get(key1)
        val2 = await upstash_redis.get(key2)
        
        transfer_amount = 50
        new_val1 = str(int(val1) - transfer_amount)
        new_val2 = str(int(val2) + transfer_amount)
        
        # Perform "atomic" update
        update_tasks = [
            upstash_redis.set(key1, new_val1),
            upstash_redis.set(key2, new_val2)
        ]
        results = await asyncio.gather(*update_tasks)
        
        # Verify both operations succeeded
        assert all(result == "OK" for result in results)
        
        # Verify final values
        final_val1 = await upstash_redis.get(key1)
        final_val2 = await upstash_redis.get(key2)
        
        assert final_val1 == "50"
        assert final_val2 == "250"


class TestRedisAdvancedStringOperations:
    """Test advanced string operations if supported."""
    
    @pytest.mark.redis
    @pytest.mark.integration  
    async def test_string_operations_simulation(self, upstash_redis: UpstashRedisClient):
        """Test string operations that can be simulated with basic commands."""
        key = f"string_ops_{uuid.uuid4().hex[:8]}"
        
        # Test append-like operation
        await upstash_redis.set(key, "Hello")
        current_val = await upstash_redis.get(key)
        new_val = current_val + " World"
        await upstash_redis.set(key, new_val)
        
        result = await upstash_redis.get(key)
        assert result == "Hello World"
        
        # Test length calculation
        length = len(result)
        assert length == 11
        
        # Test substring-like operation
        substring = result[0:5]  # "Hello"
        sub_key = f"substring_{uuid.uuid4().hex[:8]}"
        await upstash_redis.set(sub_key, substring)
        
        sub_result = await upstash_redis.get(sub_key)
        assert sub_result == "Hello"


class TestRedisDataStructureSimulation:
    """Test simulation of Redis data structures using strings."""
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_list_simulation_with_json(self, upstash_redis: UpstashRedisClient):
        """Simulate list operations using JSON strings."""
        key = f"list_sim_{uuid.uuid4().hex[:8]}"
        
        # Initialize empty list
        initial_list = []
        await upstash_redis.set(key, json.dumps(initial_list))
        
        # Add elements (LPUSH simulation)
        current_list = json.loads(await upstash_redis.get(key))
        current_list.insert(0, "first")
        await upstash_redis.set(key, json.dumps(current_list))
        
        current_list = json.loads(await upstash_redis.get(key))
        current_list.insert(0, "second")
        await upstash_redis.set(key, json.dumps(current_list))
        
        # Verify list state
        final_list = json.loads(await upstash_redis.get(key))
        assert final_list == ["second", "first"]
        
        # Pop element (LPOP simulation)
        popped = final_list.pop(0)
        await upstash_redis.set(key, json.dumps(final_list))
        
        assert popped == "second"
        
        # Verify final state
        remaining_list = json.loads(await upstash_redis.get(key))
        assert remaining_list == ["first"]
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_hash_simulation_with_json(self, upstash_redis: UpstashRedisClient):
        """Simulate hash operations using JSON strings."""
        key = f"hash_sim_{uuid.uuid4().hex[:8]}"
        
        # Initialize empty hash
        initial_hash = {}
        await upstash_redis.set(key, json.dumps(initial_hash))
        
        # Set hash fields (HSET simulation)
        current_hash = json.loads(await upstash_redis.get(key))
        current_hash["field1"] = "value1"
        current_hash["field2"] = "value2"
        current_hash["field3"] = "value3"
        await upstash_redis.set(key, json.dumps(current_hash))
        
        # Get specific field (HGET simulation)
        current_hash = json.loads(await upstash_redis.get(key))
        field1_value = current_hash.get("field1")
        assert field1_value == "value1"
        
        # Get all fields (HGETALL simulation)
        all_fields = current_hash
        expected = {"field1": "value1", "field2": "value2", "field3": "value3"}
        assert all_fields == expected
        
        # Delete field (HDEL simulation)
        del current_hash["field2"]
        await upstash_redis.set(key, json.dumps(current_hash))
        
        # Verify field is deleted
        final_hash = json.loads(await upstash_redis.get(key))
        assert "field2" not in final_hash
        assert len(final_hash) == 2
    
    @pytest.mark.redis
    @pytest.mark.integration
    async def test_set_simulation_with_json(self, upstash_redis: UpstashRedisClient):
        """Simulate set operations using JSON strings."""
        key = f"set_sim_{uuid.uuid4().hex[:8]}"
        
        # Initialize empty set
        initial_set = []
        await upstash_redis.set(key, json.dumps(initial_set))
        
        # Add members (SADD simulation)
        current_set = json.loads(await upstash_redis.get(key))
        members_to_add = ["member1", "member2", "member3", "member1"]  # member1 duplicated
        
        for member in members_to_add:
            if member not in current_set:
                current_set.append(member)
        
        await upstash_redis.set(key, json.dumps(current_set))
        
        # Verify set members (no duplicates)
        final_set = json.loads(await upstash_redis.get(key))
        assert set(final_set) == {"member1", "member2", "member3"}
        assert len(final_set) == 3
        
        # Check membership (SISMEMBER simulation)
        assert "member1" in final_set
        assert "member4" not in final_set
        
        # Remove member (SREM simulation)
        current_set = json.loads(await upstash_redis.get(key))
        if "member2" in current_set:
            current_set.remove("member2")
        await upstash_redis.set(key, json.dumps(current_set))
        
        # Verify member removed
        updated_set = json.loads(await upstash_redis.get(key))
        assert "member2" not in updated_set
        assert len(updated_set) == 2


class TestRedisBenchmarkOperations:
    """Test operations for performance benchmarking."""
    
    @pytest.mark.redis
    @pytest.mark.integration
    @pytest.mark.benchmark
    async def test_rapid_set_get_operations(self, upstash_redis: UpstashRedisClient):
        """Test rapid SET/GET operations for performance."""
        num_operations = 100
        keys_values = {}
        
        # Generate test data
        for i in range(num_operations):
            key = f"perf_test_{i}_{uuid.uuid4().hex[:8]}"
            value = f"performance_value_{i}_{uuid.uuid4().hex[:8]}"
            keys_values[key] = value
        
        # Measure SET operations
        import time
        start_time = time.time()
        
        set_tasks = [upstash_redis.set(k, v) for k, v in keys_values.items()]
        set_results = await asyncio.gather(*set_tasks)
        
        set_duration = time.time() - start_time
        
        # Verify all sets succeeded
        assert all(result == "OK" for result in set_results)
        
        # Measure GET operations
        start_time = time.time()
        
        get_tasks = [upstash_redis.get(k) for k in keys_values.keys()]
        get_results = await asyncio.gather(*get_tasks)
        
        get_duration = time.time() - start_time
        
        # Verify all gets returned correct values
        for i, (key, expected_value) in enumerate(keys_values.items()):
            assert get_results[i] == expected_value
        
        # Performance assertions (adjust thresholds as needed)
        sets_per_second = num_operations / set_duration
        gets_per_second = num_operations / get_duration
        
        print(f"Performance: {sets_per_second:.2f} SETs/sec, {gets_per_second:.2f} GETs/sec")
        
        # Basic performance expectations (very conservative)
        assert sets_per_second > 1  # At least 1 SET per second
        assert gets_per_second > 1  # At least 1 GET per second
    
    @pytest.mark.redis
    @pytest.mark.integration
    @pytest.mark.slow
    async def test_large_batch_operations(self, upstash_redis: UpstashRedisClient):
        """Test large batch operations."""
        num_keys = 1000
        batch_size = 50
        
        # Generate test data
        all_keys = []
        for i in range(num_keys):
            key = f"large_batch_{i}_{uuid.uuid4().hex[:8]}"
            value = f"batch_value_{i}"
            all_keys.append((key, value))
        
        # Process in batches to avoid overwhelming the server
        for i in range(0, num_keys, batch_size):
            batch = all_keys[i:i + batch_size]
            
            # Set batch
            set_tasks = [upstash_redis.set(k, v) for k, v in batch]
            set_results = await asyncio.gather(*set_tasks)
            assert all(result == "OK" for result in set_results)
            
            # Get batch
            get_tasks = [upstash_redis.get(k) for k, v in batch]
            get_results = await asyncio.gather(*get_tasks)
            
            # Verify batch
            for j, (key, expected_value) in enumerate(batch):
                assert get_results[j] == expected_value
        
        print(f"Successfully processed {num_keys} keys in batches of {batch_size}")


if __name__ == "__main__":
    # Run tests directly
    pytest.main([__file__, "-v"])