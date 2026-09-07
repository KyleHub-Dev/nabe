INSERT OR IGNORE INTO permissions(id, description, scope, created_at) VALUES
  ('edge.read', 'Read edge nodes and engine state.', 'global', '1970-01-01T00:00:00Z'),
  ('edge.manage', 'Enroll and manage edge nodes and engine state.', 'global', '1970-01-01T00:00:00Z'),
  ('stats.read_tenant', 'Read tenant-scoped aggregated DNS statistics.', 'tenant', '1970-01-01T00:00:00Z'),
  ('querylog.read_tenant', 'Read tenant and Device Client filtered DNS query logs.', 'tenant', '1970-01-01T00:00:00Z'),
  ('audit.read', 'Read scoped control-plane audit events.', 'global', '1970-01-01T00:00:00Z'),
  ('policy.read', 'Read DNS policies.', 'global', '1970-01-01T00:00:00Z'),
  ('policy.manage', 'Create, update, and apply DNS policies.', 'global', '1970-01-01T00:00:00Z');

INSERT OR IGNORE INTO global_role_permissions(role_id, permission_id, created_at) VALUES
  ('admin', 'edge.read', '1970-01-01T00:00:00Z'),
  ('admin', 'edge.manage', '1970-01-01T00:00:00Z'),
  ('admin', 'stats.read_tenant', '1970-01-01T00:00:00Z'),
  ('admin', 'querylog.read_tenant', '1970-01-01T00:00:00Z'),
  ('admin', 'audit.read', '1970-01-01T00:00:00Z'),
  ('admin', 'policy.read', '1970-01-01T00:00:00Z'),
  ('admin', 'policy.manage', '1970-01-01T00:00:00Z');

-- Preserve authorization for installations created before native grants were
-- enforced. New subjects are baseline-only until explicitly granted.
INSERT OR IGNORE INTO subject_global_roles(subject_id, role_id, created_at)
  SELECT id, 'admin', last_seen_at FROM oidc_subjects WHERE role = 'admin';
