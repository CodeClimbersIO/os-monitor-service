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
  app_id TEXT NOT NULL,
  platform TEXT NOT NULL CHECK (platform IN ('MAC', 'WINDOWS', 'LINUX', 'IOS', 'ANDROID')) DEFAULT 'MAC',
  is_browser BOOLEAN NOT NULL DEFAULT FALSE,
  is_default BOOLEAN NOT NULL DEFAULT FALSE,
  is_blocked BOOLEAN NOT NULL DEFAULT FALSE,
  created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
  UNIQUE(name, app_id, platform)
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

INSERT INTO tag (id, name, tag_type, is_default) VALUES ('38c0b705-842c-4121-bff8-4fac033f1a4b', 'idle', 'default', TRUE);