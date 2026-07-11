CREATE TABLE IF NOT EXISTS device_clients (
  id TEXT PRIMARY KEY,
  label TEXT NOT NULL,
  owner_scope TEXT NOT NULL DEFAULT 'default',
  client_id TEXT NOT NULL UNIQUE,
  enabled INTEGER NOT NULL DEFAULT 1,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS dns_policies (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  blocked_domains_json TEXT NOT NULL DEFAULT '[]',
  enabled INTEGER NOT NULL DEFAULT 1,
  last_applied_at TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
