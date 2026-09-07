CREATE TABLE IF NOT EXISTS telemetry_ingestion_batches (
  id TEXT PRIMARY KEY,
  edge_node_id TEXT NOT NULL,
  collected_through TEXT NOT NULL,
  created_at TEXT NOT NULL,
  FOREIGN KEY (edge_node_id) REFERENCES edge_nodes(id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS dns_stat_buckets (
  edge_node_id TEXT NOT NULL,
  device_client_id TEXT NOT NULL,
  bucket_start TEXT NOT NULL,
  bucket_seconds INTEGER NOT NULL,
  queries INTEGER NOT NULL DEFAULT 0,
  blocked INTEGER NOT NULL DEFAULT 0,
  cached INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT NOT NULL,
  PRIMARY KEY (edge_node_id, device_client_id, bucket_start, bucket_seconds),
  FOREIGN KEY (edge_node_id) REFERENCES edge_nodes(id) ON DELETE CASCADE,
  FOREIGN KEY (device_client_id) REFERENCES device_clients(id) ON DELETE CASCADE,
  CHECK (bucket_seconds = 300),
  CHECK (queries >= 0 AND blocked >= 0 AND cached >= 0),
  CHECK (blocked <= queries AND cached <= queries)
);

CREATE INDEX IF NOT EXISTS dns_stat_buckets_time_idx
  ON dns_stat_buckets(bucket_start DESC);
CREATE INDEX IF NOT EXISTS dns_stat_buckets_device_time_idx
  ON dns_stat_buckets(device_client_id, bucket_start DESC);
