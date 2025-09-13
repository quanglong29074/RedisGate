-- Create Redis instances table
-- This table stores Redis instance configurations and metadata

CREATE TABLE redis_instances (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    slug VARCHAR(255) NOT NULL,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    api_key_id UUID NOT NULL REFERENCES api_keys(id) ON DELETE CASCADE,
    
    -- Network configuration
    port INTEGER NOT NULL,
    private_ip_address INET,
    public_ip_address INET,
    domain VARCHAR(255) UNIQUE,
    
    -- Redis configuration
    max_memory BIGINT NOT NULL DEFAULT 134217728, -- 128MB default
    current_memory BIGINT DEFAULT 0,
    password_hash VARCHAR(255),
    redis_version VARCHAR(20) DEFAULT 'latest',
    
    -- Kubernetes configuration
    namespace VARCHAR(255) NOT NULL,
    pod_name VARCHAR(255),
    service_name VARCHAR(255),
    
    -- Instance status and metadata
    status VARCHAR(50) NOT NULL DEFAULT 'pending', -- pending, creating, running, stopping, stopped, error
    last_health_check_at TIMESTAMP WITH TIME ZONE,
    health_status VARCHAR(50) DEFAULT 'unknown', -- healthy, unhealthy, unknown
    
    -- Resource usage tracking
    cpu_usage_percent DECIMAL(5,2) DEFAULT 0.0,
    memory_usage_percent DECIMAL(5,2) DEFAULT 0.0,
    connections_count INTEGER DEFAULT 0,
    max_connections INTEGER DEFAULT 1000,
    
    -- Backup and persistence
    persistence_enabled BOOLEAN DEFAULT TRUE,
    backup_enabled BOOLEAN DEFAULT FALSE,
    last_backup_at TIMESTAMP WITH TIME ZONE,
    
    -- Timestamps
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    deleted_at TIMESTAMP WITH TIME ZONE -- For soft deletion
);

-- Create indexes for performance
CREATE INDEX idx_redis_instances_organization_id ON redis_instances(organization_id);
CREATE INDEX idx_redis_instances_api_key_id ON redis_instances(api_key_id);
CREATE INDEX idx_redis_instances_status ON redis_instances(status);
CREATE INDEX idx_redis_instances_domain ON redis_instances(domain);
CREATE INDEX idx_redis_instances_namespace ON redis_instances(namespace);
CREATE INDEX idx_redis_instances_health_status ON redis_instances(health_status);
CREATE INDEX idx_redis_instances_deleted_at ON redis_instances(deleted_at);

-- Create unique constraint for organization + slug
CREATE UNIQUE INDEX idx_redis_instances_org_slug ON redis_instances(organization_id, slug) WHERE deleted_at IS NULL;
