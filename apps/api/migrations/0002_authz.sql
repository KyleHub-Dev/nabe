CREATE TABLE IF NOT EXISTS permissions (
  id TEXT PRIMARY KEY,
  description TEXT NOT NULL,
  scope TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS global_roles (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL UNIQUE,
  description TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS global_role_permissions (
  role_id TEXT NOT NULL,
  permission_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (role_id, permission_id),
  FOREIGN KEY (role_id) REFERENCES global_roles(id) ON DELETE CASCADE,
  FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS subject_global_roles (
  subject_id TEXT NOT NULL,
  role_id TEXT NOT NULL,
  granted_by_subject_id TEXT,
  created_at TEXT NOT NULL,
  PRIMARY KEY (subject_id, role_id),
  FOREIGN KEY (subject_id) REFERENCES oidc_subjects(id) ON DELETE CASCADE,
  FOREIGN KEY (role_id) REFERENCES global_roles(id) ON DELETE CASCADE,
  FOREIGN KEY (granted_by_subject_id) REFERENCES oidc_subjects(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS tenants (
  id TEXT PRIMARY KEY,
  slug TEXT NOT NULL UNIQUE,
  name TEXT NOT NULL,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_roles (
  id TEXT PRIMARY KEY,
  tenant_id TEXT NOT NULL,
  name TEXT NOT NULL,
  description TEXT NOT NULL,
  created_at TEXT NOT NULL,
  UNIQUE (tenant_id, name),
  FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS tenant_role_templates (
  name TEXT PRIMARY KEY,
  description TEXT NOT NULL,
  created_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS tenant_role_template_permissions (
  role_name TEXT NOT NULL,
  permission_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (role_name, permission_id),
  FOREIGN KEY (role_name) REFERENCES tenant_role_templates(name) ON DELETE CASCADE,
  FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS tenant_role_permissions (
  role_id TEXT NOT NULL,
  permission_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (role_id, permission_id),
  FOREIGN KEY (role_id) REFERENCES tenant_roles(id) ON DELETE CASCADE,
  FOREIGN KEY (permission_id) REFERENCES permissions(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS tenant_memberships (
  tenant_id TEXT NOT NULL,
  subject_id TEXT NOT NULL,
  role_id TEXT NOT NULL,
  granted_by_subject_id TEXT,
  created_at TEXT NOT NULL,
  PRIMARY KEY (tenant_id, subject_id, role_id),
  FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
  FOREIGN KEY (subject_id) REFERENCES oidc_subjects(id) ON DELETE CASCADE,
  FOREIGN KEY (role_id) REFERENCES tenant_roles(id) ON DELETE CASCADE,
  FOREIGN KEY (granted_by_subject_id) REFERENCES oidc_subjects(id) ON DELETE SET NULL
);

CREATE TABLE IF NOT EXISTS tenant_dns_engine_instances (
  tenant_id TEXT NOT NULL,
  dns_engine_instance_id TEXT NOT NULL,
  created_at TEXT NOT NULL,
  PRIMARY KEY (tenant_id, dns_engine_instance_id),
  FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
  FOREIGN KEY (dns_engine_instance_id) REFERENCES dns_engine_instances(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS subject_device_clients (
  id TEXT PRIMARY KEY,
  subject_id TEXT NOT NULL,
  tenant_id TEXT,
  dns_engine_instance_id TEXT NOT NULL,
  name TEXT NOT NULL,
  client_id TEXT NOT NULL UNIQUE,
  status TEXT NOT NULL DEFAULT 'active',
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  revoked_at TEXT,
  FOREIGN KEY (subject_id) REFERENCES oidc_subjects(id) ON DELETE CASCADE,
  FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE SET NULL,
  FOREIGN KEY (dns_engine_instance_id) REFERENCES dns_engine_instances(id) ON DELETE CASCADE
);

INSERT OR IGNORE INTO permissions(id, description, scope, created_at) VALUES
  ('platform.admin', 'Full Nabe platform administration.', 'global', '1970-01-01T00:00:00Z'),
  ('cloud_dns.use', 'Use Cloud DNS without requiring tenant membership.', 'global', '1970-01-01T00:00:00Z'),
  ('device_client.read_own', 'Read own Device Clients.', 'global', '1970-01-01T00:00:00Z'),
  ('device_client.manage_own', 'Manage own Device Clients.', 'global', '1970-01-01T00:00:00Z'),
  ('tenant.read', 'Read tenant resources.', 'tenant', '1970-01-01T00:00:00Z'),
  ('tenant.manage', 'Manage tenant settings and members.', 'tenant', '1970-01-01T00:00:00Z'),
  ('tenant.device_client.read', 'Read tenant Device Clients.', 'tenant', '1970-01-01T00:00:00Z'),
  ('tenant.device_client.manage', 'Manage tenant Device Clients.', 'tenant', '1970-01-01T00:00:00Z'),
  ('tenant.querylog.read', 'Read tenant-filtered query logs.', 'tenant', '1970-01-01T00:00:00Z'),
  ('engine.read', 'Read DNS Engine status and statistics.', 'global', '1970-01-01T00:00:00Z'),
  ('engine.manage', 'Manage DNS Engine configuration.', 'global', '1970-01-01T00:00:00Z');

INSERT OR IGNORE INTO global_roles(id, name, description, created_at) VALUES
  ('admin', 'admin', 'Platform administrator for Nabe.', '1970-01-01T00:00:00Z'),
  ('authenticated', 'authenticated', 'Baseline role for authenticated users without tenant membership.', '1970-01-01T00:00:00Z');

INSERT OR IGNORE INTO global_role_permissions(role_id, permission_id, created_at) VALUES
  ('admin', 'platform.admin', '1970-01-01T00:00:00Z'),
  ('admin', 'cloud_dns.use', '1970-01-01T00:00:00Z'),
  ('admin', 'device_client.read_own', '1970-01-01T00:00:00Z'),
  ('admin', 'device_client.manage_own', '1970-01-01T00:00:00Z'),
  ('admin', 'engine.read', '1970-01-01T00:00:00Z'),
  ('admin', 'engine.manage', '1970-01-01T00:00:00Z'),
  ('authenticated', 'cloud_dns.use', '1970-01-01T00:00:00Z'),
  ('authenticated', 'device_client.read_own', '1970-01-01T00:00:00Z'),
  ('authenticated', 'device_client.manage_own', '1970-01-01T00:00:00Z');

INSERT OR IGNORE INTO tenant_role_templates(name, description, created_at) VALUES
  ('manager', 'Tenant administrator. Can manage tenant settings, members, Device Clients, and tenant query logs.', '1970-01-01T00:00:00Z'),
  ('user', 'Tenant user. Can use tenant DNS resources and manage own Device Clients.', '1970-01-01T00:00:00Z'),
  ('viewer', 'Read-only tenant member. Can inspect tenant resources and query logs without making changes.', '1970-01-01T00:00:00Z');

INSERT OR IGNORE INTO tenant_role_template_permissions(role_name, permission_id, created_at) VALUES
  ('manager', 'tenant.read', '1970-01-01T00:00:00Z'),
  ('manager', 'tenant.manage', '1970-01-01T00:00:00Z'),
  ('manager', 'tenant.device_client.read', '1970-01-01T00:00:00Z'),
  ('manager', 'tenant.device_client.manage', '1970-01-01T00:00:00Z'),
  ('manager', 'tenant.querylog.read', '1970-01-01T00:00:00Z'),
  ('user', 'tenant.read', '1970-01-01T00:00:00Z'),
  ('user', 'tenant.device_client.read', '1970-01-01T00:00:00Z'),
  ('user', 'tenant.device_client.manage', '1970-01-01T00:00:00Z'),
  ('viewer', 'tenant.read', '1970-01-01T00:00:00Z'),
  ('viewer', 'tenant.device_client.read', '1970-01-01T00:00:00Z'),
  ('viewer', 'tenant.querylog.read', '1970-01-01T00:00:00Z');
