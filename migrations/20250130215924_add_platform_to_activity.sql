-- Add migration script here
ALTER TABLE activity ADD COLUMN platform TEXT NOT NULL CHECK (platform IN ('MAC', 'WINDOWS', 'LINUX', 'IOS', 'ANDROID', 'UNKNOWN')) DEFAULT 'MAC';
