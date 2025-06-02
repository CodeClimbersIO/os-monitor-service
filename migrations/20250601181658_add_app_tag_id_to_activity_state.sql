CREATE TABLE IF NOT EXISTS activity_state_tag_new (
  activity_state_id TEXT NOT NULL,
  tag_id TEXT NOT NULL,
  app_tag_id TEXT,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (activity_state_id) REFERENCES activity_state(id),
  FOREIGN KEY (tag_id) REFERENCES tag(id),
  FOREIGN KEY (app_tag_id) REFERENCES app_tag(id)
);

-- Copy existing data from the old table
INSERT INTO activity_state_tag_new (activity_state_id, tag_id, created_at, updated_at)
SELECT activity_state_id, tag_id, created_at, updated_at
FROM activity_state_tag;

-- Rename the old table
ALTER TABLE activity_state_tag RENAME TO activity_state_tag_old;

-- Rename the new table
ALTER TABLE activity_state_tag_new RENAME TO activity_state_tag;

-- Add the new unique constraint
CREATE UNIQUE INDEX IF NOT EXISTS idx_activity_state_tag_app_tag_unique_id
  ON activity_state_tag(tag_id, app_tag_id, activity_state_id);