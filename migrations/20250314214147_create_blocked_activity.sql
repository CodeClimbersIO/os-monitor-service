CREATE TABLE IF NOT EXISTS blocked_activity (
  id TEXT PRIMARY KEY NOT NULL,
  external_app_id TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_blocked_activity_external_app_id ON blocked_activity(external_app_id);