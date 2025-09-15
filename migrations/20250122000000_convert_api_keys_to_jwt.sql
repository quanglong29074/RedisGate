-- Convert API keys from bcrypt hash to JWT tokens
-- This migration replaces the key_hash column with key_token to store JWT tokens directly

-- First, add the new key_token column
ALTER TABLE api_keys ADD COLUMN key_token TEXT;

-- Copy existing key_hash values to key_token temporarily (for migration safety)
-- Note: In production, this would require regenerating all API keys as JWTs
-- For now, we'll clear existing keys and require regeneration
UPDATE api_keys SET key_token = 'MIGRATION_REQUIRED';

-- Make key_token required and unique
ALTER TABLE api_keys ALTER COLUMN key_token SET NOT NULL;
CREATE UNIQUE INDEX idx_api_keys_token ON api_keys(key_token);

-- Remove the old key_hash column and its index
DROP INDEX IF EXISTS idx_api_keys_hash;
ALTER TABLE api_keys DROP COLUMN key_hash;

-- Update the existing unique index on key_hash to use key_token instead
-- (This is now redundant with idx_api_keys_token but kept for reference)