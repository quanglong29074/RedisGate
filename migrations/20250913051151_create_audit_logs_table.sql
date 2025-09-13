-- Create audit logs table
-- This table tracks important actions and changes for security and compliance

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    organization_id UUID REFERENCES organizations(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL, -- create, update, delete, login, logout, etc.
    resource_type VARCHAR(50) NOT NULL, -- user, organization, redis_instance, api_key, etc.
    resource_id UUID,
    details JSONB, -- Additional context and metadata
    ip_address INET,
    user_agent TEXT,
    api_key_id UUID REFERENCES api_keys(id) ON DELETE SET NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'success', -- success, failure, pending
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for performance and querying
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_organization_id ON audit_logs(organization_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_resource_type ON audit_logs(resource_type);
CREATE INDEX idx_audit_logs_resource_id ON audit_logs(resource_id);
CREATE INDEX idx_audit_logs_api_key_id ON audit_logs(api_key_id);
CREATE INDEX idx_audit_logs_status ON audit_logs(status);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at);

-- Create composite index for common queries
CREATE INDEX idx_audit_logs_org_created ON audit_logs(organization_id, created_at DESC);
