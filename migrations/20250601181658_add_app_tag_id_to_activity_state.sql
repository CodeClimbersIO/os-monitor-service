ALTER TABLE activity_state_tag ADD COLUMN app_tag_id TEXT REFERENCES app_tag(id);

ALTER TABLE activity_state_tag DROP INDEX
CREATE UNIQUE INDEX idx_activity_state_tag ON activity_state_tag(activity_state_id, app_tag_id);

--ROLLBACK
-- ALTER TABLE activity_state_tag DROP COLUMN app_tag_id;
