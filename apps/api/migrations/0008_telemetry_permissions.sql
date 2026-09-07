INSERT OR IGNORE INTO permissions(id, description, scope, created_at) VALUES
  ('querylog.read_own', 'Read raw query logs for owned Device Clients.', 'global', '1970-01-01T00:00:00Z'),
  ('stats.read_own', 'Read aggregated statistics for owned Device Clients.', 'global', '1970-01-01T00:00:00Z');

INSERT OR IGNORE INTO permissions(id, description, scope, created_at) VALUES
  ('tenant.stats.read', 'Read aggregated statistics for a tenant.', 'tenant', '1970-01-01T00:00:00Z');

INSERT OR IGNORE INTO global_role_permissions(role_id, permission_id, created_at) VALUES
  ('admin', 'querylog.read_own', '1970-01-01T00:00:00Z'),
  ('admin', 'stats.read_own', '1970-01-01T00:00:00Z'),
  ('baseline', 'querylog.read_own', '1970-01-01T00:00:00Z'),
  ('baseline', 'stats.read_own', '1970-01-01T00:00:00Z');

INSERT OR IGNORE INTO tenant_role_template_permissions(role_name, permission_id, created_at) VALUES
  ('manager', 'tenant.stats.read', '1970-01-01T00:00:00Z'),
  ('user', 'tenant.stats.read', '1970-01-01T00:00:00Z'),
  ('viewer', 'tenant.stats.read', '1970-01-01T00:00:00Z');

INSERT OR IGNORE INTO tenant_role_permissions(role_id, permission_id, created_at)
  SELECT tr.id, template.permission_id, '1970-01-01T00:00:00Z'
  FROM tenant_roles tr
  JOIN tenant_role_template_permissions template ON template.role_name = tr.name;
