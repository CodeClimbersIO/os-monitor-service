CREATE TABLE IF NOT EXISTS tag (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT NOT NULL,
  parent_tag_id TEXT,
  tag_type TEXT NOT NULL,
  is_blocked BOOLEAN NOT NULL DEFAULT FALSE,
  is_default BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (parent_tag_id) REFERENCES tag(id),
  UNIQUE(name, tag_type)
);

CREATE TABLE IF NOT EXISTS activity_state_tag (
  activity_state_id TEXT NOT NULL,
  tag_id TEXT NOT NULL,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (activity_state_id) REFERENCES activity_state(id),
  FOREIGN KEY (tag_id) REFERENCES tag(id),
  UNIQUE(activity_state_id, tag_id)
);

CREATE TABLE IF NOT EXISTS app (
  id TEXT PRIMARY KEY NOT NULL,
  name TEXT,
  app_external_id TEXT NOT NULL,
  platform TEXT NOT NULL CHECK (platform IN ('MAC', 'WINDOWS', 'LINUX', 'IOS', 'ANDROID')) DEFAULT 'MAC',
  is_browser BOOLEAN NOT NULL DEFAULT FALSE,
  is_default BOOLEAN NOT NULL DEFAULT FALSE,
  is_blocked BOOLEAN NOT NULL DEFAULT FALSE,
  metadata TEXT, -- JSON string containing the bundle_id, app_name, and url
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(app_external_id, platform)
);

CREATE TABLE IF NOT EXISTS app_tag (
  id TEXT PRIMARY KEY NOT NULL,
  app_id TEXT NOT NULL,
  tag_id TEXT NOT NULL,
  weight REAL NOT NULL,
  is_default BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  FOREIGN KEY (app_id) REFERENCES app(id),
  FOREIGN KEY (tag_id) REFERENCES tag(id),
  UNIQUE(app_id, tag_id)
);

ALTER TABLE activity ADD COLUMN platform TEXT NOT NULL CHECK (platform IN ('MAC', 'WINDOWS', 'LINUX', 'IOS', 'ANDROID', 'UNKNOWN')) DEFAULT 'MAC';

ALTER TABLE activity ADD COLUMN app_id TEXT REFERENCES app(id);

-- app_id will now be used to identify the activities source. 
ALTER TABLE activity DROP COLUMN app_name;
ALTER TABLE activity DROP COLUMN url;

-- Add index for timestamp lookups
CREATE INDEX idx_activity_timestamp ON activity(timestamp);

-- Add index for activity_state date range queries
CREATE INDEX idx_activity_state_times ON activity_state(start_time, end_time);