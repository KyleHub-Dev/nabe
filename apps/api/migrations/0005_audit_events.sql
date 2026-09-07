ALTER TABLE audit_events ADD COLUMN tenant_id TEXT;
ALTER TABLE audit_events ADD COLUMN outcome TEXT NOT NULL DEFAULT 'allowed';
ALTER TABLE audit_events ADD COLUMN reason TEXT;
ALTER TABLE audit_events ADD COLUMN request_id TEXT;

CREATE INDEX IF NOT EXISTS audit_events_created_at_idx
  ON audit_events(created_at DESC);
CREATE INDEX IF NOT EXISTS audit_events_actor_idx
  ON audit_events(actor_provider, actor_subject, created_at DESC);
CREATE INDEX IF NOT EXISTS audit_events_tenant_idx
  ON audit_events(tenant_id, created_at DESC);
CREATE INDEX IF NOT EXISTS audit_events_action_idx
  ON audit_events(action, created_at DESC);
