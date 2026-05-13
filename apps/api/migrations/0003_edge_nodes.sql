CREATE TABLE IF NOT EXISTS edge_nodes (
  id TEXT PRIMARY KEY,
  name TEXT NOT NULL,
  hostname TEXT NOT NULL,
  architecture TEXT NOT NULL,
  os TEXT NOT NULL,
  kernel TEXT NOT NULL,
  speiche_version TEXT NOT NULL,
  health_status TEXT NOT NULL DEFAULT 'enrolling',
  inventory_json TEXT,
  enrolled_at TEXT NOT NULL,
  last_seen_at TEXT NOT NULL,
  updated_at TEXT NOT NULL
);
