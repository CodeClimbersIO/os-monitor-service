-- Add migration script here
UPDATE app SET app_external_id = 'com.apple.TV' WHERE id = '242dbf5a-efca-47b6-bb6f-a36ecdc79410';

INSERT INTO app (id, name, app_external_id, platform, is_browser, is_default) VALUES
('b6791fe7-909b-4491-8554-6c00db38575e', 'Terminal', 'com.apple.Terminal', 'MAC', 0, TRUE);

INSERT INTO app_tag (id, app_id, tag_id, weight, is_default) VALUES
('f686fa17-2e1c-4f66-9111-0f909c911b6d', 'b6791fe7-909b-4491-8554-6c00db38575e', '5ba10e13-d342-4262-a391-9b9aa95332cd', 1.0, TRUE);