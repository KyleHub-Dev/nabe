CREATE TABLE IF NOT EXISTS edge_node_credentials (
  edge_node_id TEXT PRIMARY KEY,
  token_hash TEXT NOT NULL UNIQUE,
  created_at TEXT NOT NULL,
  last_used_at TEXT,
  revoked_at TEXT,
  FOREIGN KEY (edge_node_id) REFERENCES edge_nodes(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS edge_node_credentials_hash_idx
  ON edge_node_credentials(token_hash) WHERE revoked_at IS NULL;
