-- Make api_key_id nullable in redis_instances table
-- This allows creating Redis instances before API keys

ALTER TABLE redis_instances ALTER COLUMN api_key_id DROP NOT NULL;
