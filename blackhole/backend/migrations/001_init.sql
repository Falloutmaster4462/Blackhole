-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================================
-- ENUMS
-- ============================================================================

CREATE TYPE email_state AS ENUM (
    'NEW',
    'CLAIMED',
    'RESPONDED',
    'RESOLVED',
    'ARCHIVED'
);

CREATE TYPE user_role AS ENUM (
    'ADMIN',
    'AGENT',
    'VIEWER'
);

-- ============================================================================
-- TENANTS TABLE
-- ============================================================================

CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    domain VARCHAR(255) NOT NULL UNIQUE,
    settings JSONB DEFAULT '{}',
    storage_used_bytes BIGINT DEFAULT 0,
    storage_limit_bytes BIGINT DEFAULT 10737418240, -- 10GB default
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_tenants_domain ON tenants(domain);
CREATE INDEX idx_tenants_is_active ON tenants(is_active);

-- ============================================================================
-- USERS TABLE
-- ============================================================================

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    role user_role DEFAULT 'AGENT',
    is_active BOOLEAN DEFAULT TRUE,
    last_login_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(tenant_id, email)
);

CREATE INDEX idx_users_tenant_id ON users(tenant_id);
CREATE INDEX idx_users_email ON users(email);
CREATE INDEX idx_users_is_active ON users(is_active);

-- ============================================================================
-- INBOXES TABLE
-- ============================================================================

CREATE TABLE inboxes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    email_address VARCHAR(255) NOT NULL,
    description TEXT,
    is_shared BOOLEAN DEFAULT FALSE,
    color VARCHAR(7), -- Hex color
    icon VARCHAR(50),
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(tenant_id, email_address)
);

CREATE INDEX idx_inboxes_tenant_id ON inboxes(tenant_id);
CREATE INDEX idx_inboxes_email_address ON inboxes(email_address);
CREATE INDEX idx_inboxes_is_shared ON inboxes(is_shared);

-- ============================================================================
-- EMAILS TABLE
-- ============================================================================

CREATE TABLE emails (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    inbox_id UUID REFERENCES inboxes(id) ON DELETE SET NULL,
    message_id VARCHAR(255) NOT NULL,
    in_reply_to VARCHAR(255),
    thread_id UUID NOT NULL,
    
    -- Headers
    from_address VARCHAR(255) NOT NULL,
    from_name VARCHAR(255),
    to_addresses TEXT[] NOT NULL,
    cc_addresses TEXT[],
    bcc_addresses TEXT[],
    subject TEXT NOT NULL,
    
    -- Content
    body_text TEXT,
    body_html TEXT,
    headers JSONB DEFAULT '{}',
    
    -- Metadata
    size_bytes BIGINT NOT NULL,
    has_attachments BOOLEAN DEFAULT FALSE,
    attachment_count INTEGER DEFAULT 0,
    
    -- Lifecycle
    state email_state DEFAULT 'NEW',
    assigned_to UUID REFERENCES users(id) ON DELETE SET NULL,
    tags TEXT[] DEFAULT '{}',
    
    -- SLA & Timing
    sla_deadline TIMESTAMP WITH TIME ZONE,
    first_response_at TIMESTAMP WITH TIME ZONE,
    resolved_at TIMESTAMP WITH TIME ZONE,
    
    -- Timestamps
    received_at TIMESTAMP WITH TIME ZONE NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Observability
    ingest_latency_ms INTEGER,
    delivery_latency_ms INTEGER,
    
    UNIQUE(tenant_id, message_id)
);

CREATE INDEX idx_emails_tenant_id ON emails(tenant_id);
CREATE INDEX idx_emails_inbox_id ON emails(inbox_id);
CREATE INDEX idx_emails_thread_id ON emails(thread_id);
CREATE INDEX idx_emails_state ON emails(state);
CREATE INDEX idx_emails_assigned_to ON emails(assigned_to);
CREATE INDEX idx_emails_received_at ON emails(received_at DESC);
CREATE INDEX idx_emails_from_address ON emails(from_address);
CREATE INDEX idx_emails_subject ON emails USING gin(to_tsvector('english', subject));
CREATE INDEX idx_emails_body_text ON emails USING gin(to_tsvector('english', body_text));
CREATE INDEX idx_emails_tags ON emails USING gin(tags);

-- ============================================================================
-- ATTACHMENTS TABLE
-- ============================================================================

CREATE TABLE attachments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email_id UUID NOT NULL REFERENCES emails(id) ON DELETE CASCADE,
    filename VARCHAR(255) NOT NULL,
    content_type VARCHAR(255) NOT NULL,
    size_bytes BIGINT NOT NULL,
    storage_path VARCHAR(500) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_attachments_email_id ON attachments(email_id);

-- ============================================================================
-- RULES TABLE
-- ============================================================================

CREATE TABLE rules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    priority INTEGER DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    
    -- Conditions and actions stored as JSON
    conditions JSONB NOT NULL,
    actions JSONB NOT NULL,
    
    created_by UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_rules_tenant_id ON rules(tenant_id);
CREATE INDEX idx_rules_is_active ON rules(is_active);
CREATE INDEX idx_rules_priority ON rules(priority DESC);

-- ============================================================================
-- RULE EXECUTIONS TABLE
-- ============================================================================

CREATE TABLE rule_executions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email_id UUID NOT NULL REFERENCES emails(id) ON DELETE CASCADE,
    rule_id UUID NOT NULL REFERENCES rules(id) ON DELETE CASCADE,
    matched BOOLEAN NOT NULL,
    actions_taken JSONB DEFAULT '{}',
    executed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_rule_executions_email_id ON rule_executions(email_id);
CREATE INDEX idx_rule_executions_rule_id ON rule_executions(rule_id);

-- ============================================================================
-- INTERNAL NOTES TABLE
-- ============================================================================

CREATE TABLE internal_notes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email_id UUID NOT NULL REFERENCES emails(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_internal_notes_email_id ON internal_notes(email_id);
CREATE INDEX idx_internal_notes_user_id ON internal_notes(user_id);

-- ============================================================================
-- STATE HISTORY TABLE
-- ============================================================================

CREATE TABLE state_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email_id UUID NOT NULL REFERENCES emails(id) ON DELETE CASCADE,
    from_state email_state NOT NULL,
    to_state email_state NOT NULL,
    changed_by UUID REFERENCES users(id) ON DELETE SET NULL,
    reason TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_state_history_email_id ON state_history(email_id);
CREATE INDEX idx_state_history_created_at ON state_history(created_at DESC);

-- ============================================================================
-- AUDIT LOGS TABLE
-- ============================================================================

CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(255) NOT NULL,
    resource_type VARCHAR(100) NOT NULL,
    resource_id UUID,
    details JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_tenant_id ON audit_logs(tenant_id);
CREATE INDEX idx_audit_logs_user_id ON audit_logs(user_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);

-- ============================================================================
-- WEBHOOKS TABLE
-- ============================================================================

CREATE TABLE webhooks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    url VARCHAR(500) NOT NULL,
    events TEXT[] NOT NULL,
    is_active BOOLEAN DEFAULT TRUE,
    secret VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_webhooks_tenant_id ON webhooks(tenant_id);
CREATE INDEX idx_webhooks_is_active ON webhooks(is_active);

-- ============================================================================
-- TRIGGERS FOR UPDATED_AT
-- ============================================================================

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_tenants_updated_at BEFORE UPDATE ON tenants
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_inboxes_updated_at BEFORE UPDATE ON inboxes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_emails_updated_at BEFORE UPDATE ON emails
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_rules_updated_at BEFORE UPDATE ON rules
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_internal_notes_updated_at BEFORE UPDATE ON internal_notes
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- SEED DATA
-- ============================================================================

-- Default tenant
INSERT INTO tenants (id, name, domain) VALUES 
('00000000-0000-0000-0000-000000000001', 'Blackhole Demo', 'blackhole.dev');

-- Default admin user (password: admin123)
INSERT INTO users (id, tenant_id, email, name, password_hash, role) VALUES
('00000000-0000-0000-0000-000000000002', 
 '00000000-0000-0000-0000-000000000001',
 'admin@blackhole.dev',
 'Admin User',
 '$2b$12$LQv3c1yqBWVHxkd0LHAkCOYz6TtxMQJqhN8/LewY5GyYIHWNvXC2a', -- admin123
 'ADMIN');

-- Default inboxes
INSERT INTO inboxes (id, tenant_id, name, email_address, is_shared, color) VALUES
('00000000-0000-0000-0000-000000000003',
 '00000000-0000-0000-0000-000000000001',
 'Support',
 'support@blackhole.dev',
 TRUE,
 '#3B82F6'),
('00000000-0000-0000-0000-000000000004',
 '00000000-0000-0000-0000-000000000001',
 'Sales',
 'sales@blackhole.dev',
 TRUE,
 '#10B981'),
('00000000-0000-0000-0000-000000000005',
 '00000000-0000-0000-0000-000000000001',
 'Info',
 'info@blackhole.dev',
 TRUE,
 '#F59E0B');

-- Default rules
INSERT INTO rules (tenant_id, name, description, priority, conditions, actions, created_by) VALUES
('00000000-0000-0000-0000-000000000001',
 'Support Inbox Routing',
 'Route emails to support@ to the Support inbox',
 100,
 '{"to_contains": ["support@blackhole.dev"]}',
 '{"assign_inbox": "support@blackhole.dev", "add_tags": ["support"], "set_sla_hours": 24}',
 '00000000-0000-0000-0000-000000000002'),
('00000000-0000-0000-0000-000000000001',
 'Sales Inbox Routing',
 'Route emails to sales@ to the Sales inbox',
 90,
 '{"to_contains": ["sales@blackhole.dev"]}',
 '{"assign_inbox": "sales@blackhole.dev", "add_tags": ["sales"], "set_sla_hours": 48}',
 '00000000-0000-0000-0000-000000000002');
