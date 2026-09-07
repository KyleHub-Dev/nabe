ALTER TABLE device_clients ADD COLUMN owner_provider TEXT;
ALTER TABLE device_clients ADD COLUMN owner_subject TEXT;
ALTER TABLE device_clients ADD COLUMN tenant_id TEXT REFERENCES tenants(id) ON DELETE CASCADE;
ALTER TABLE dns_policies ADD COLUMN tenant_id TEXT REFERENCES tenants(id) ON DELETE CASCADE;

CREATE INDEX IF NOT EXISTS device_clients_owner_idx
  ON device_clients(owner_provider, owner_subject, created_at);
CREATE INDEX IF NOT EXISTS device_clients_tenant_idx
  ON device_clients(tenant_id, created_at);
CREATE INDEX IF NOT EXISTS dns_policies_tenant_idx
  ON dns_policies(tenant_id, created_at);

INSERT OR IGNORE INTO permissions(id, description, scope, created_at) VALUES
  ('tenant.policy.read', 'Read DNS policies in a tenant.', 'tenant', '1970-01-01T00:00:00Z'),
  ('tenant.policy.manage', 'Create, update, and apply DNS policies in a tenant.', 'tenant', '1970-01-01T00:00:00Z');

INSERT OR IGNORE INTO tenant_role_template_permissions(role_name, permission_id, created_at) VALUES
  ('manager', 'tenant.policy.read', '1970-01-01T00:00:00Z'),
  ('manager', 'tenant.policy.manage', '1970-01-01T00:00:00Z'),
  ('user', 'tenant.policy.read', '1970-01-01T00:00:00Z'),
  ('viewer', 'tenant.policy.read', '1970-01-01T00:00:00Z');

-- Existing tenant roles receive newly introduced template permissions.
INSERT OR IGNORE INTO tenant_role_permissions(role_id, permission_id, created_at)
  SELECT tr.id, template.permission_id, '1970-01-01T00:00:00Z'
  FROM tenant_roles tr
  JOIN tenant_role_template_permissions template ON template.role_name = tr.name;
