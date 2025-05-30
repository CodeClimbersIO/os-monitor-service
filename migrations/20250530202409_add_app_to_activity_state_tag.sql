-- Create a new table with the updated structure
CREATE TABLE IF NOT EXISTS activity_state_tag_new (
  activity_state_id TEXT NOT NULL,
  tag_id TEXT NOT NULL,
  app_id TEXT REFERENCES app,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (activity_state_id) REFERENCES activity_state(id),
  FOREIGN KEY (tag_id) REFERENCES tag(id)
);

-- Copy existing data from the old table
INSERT INTO activity_state_tag_new (activity_state_id, tag_id, created_at, updated_at)
SELECT activity_state_id, tag_id, created_at, updated_at
FROM activity_state_tag;

-- Drop the old table
DROP TABLE activity_state_tag;

-- Rename the new table
ALTER TABLE activity_state_tag_new RENAME TO activity_state_tag;

-- Add the new unique constraint
CREATE UNIQUE INDEX IF NOT EXISTS idx_activity_state_tag_app_tag_unique_id
  ON activity_state_tag(tag_id, app_id, activity_state_id);


-- Rollback
-- DROP INDEX IF EXISTS idx_activity_state_tag_app_tag_unique_id;
-- ALTER TABLE activity_state_tag RENAME TO activity_state_tag_old;
-- CREATE TABLE activity_state_tag AS SELECT activity_state_id, tag_id, created_at, updated_at FROM activity_state_tag_old;
-- DROP TABLE activity_state_tag_old;