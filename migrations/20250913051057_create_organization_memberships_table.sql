-- Create organization memberships table
-- This table manages user memberships in organizations (many-to-many relationship)

CREATE TABLE organization_memberships (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    organization_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL DEFAULT 'member', -- owner, admin, member, viewer
    permissions TEXT[] DEFAULT '{}', -- Array of specific permissions
    is_active BOOLEAN DEFAULT TRUE,
    invited_by UUID REFERENCES users(id),
    invitation_token VARCHAR(255),
    invitation_expires_at TIMESTAMP WITH TIME ZONE,
    joined_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for performance
CREATE INDEX idx_org_memberships_user_id ON organization_memberships(user_id);
CREATE INDEX idx_org_memberships_organization_id ON organization_memberships(organization_id);
CREATE INDEX idx_org_memberships_role ON organization_memberships(role);
CREATE INDEX idx_org_memberships_active ON organization_memberships(is_active);
CREATE INDEX idx_org_memberships_invitation_token ON organization_memberships(invitation_token);

-- Create unique constraint to prevent duplicate memberships
CREATE UNIQUE INDEX idx_org_memberships_unique ON organization_memberships(user_id, organization_id);
