-- Add migration script here
-- Add default app
INSERT INTO app (name, platform, is_browser) VALUES
('Cursor', 'MAC', 0),
('WebStorm', 'MAC', 0),
('excalidraw.com', 'MAC', 1),
('github.com', 'MAC', 1),
('0cred.com', 'MAC', 1),
('google.com', 'MAC', 1),
('stackoverflow.com', 'MAC', 1),
('Arc', 'MAC', 0),
('dash.cloudflare.com', 'MAC', 1),
('claude.ai', 'MAC', 1),
('figma.com', 'MAC', 1),
('espn.com', 'MAC', 1),
('x.com', 'MAC', 1),
('youtube.com', 'MAC', 1),
('facebook.com', 'MAC', 1),
('instagram.com', 'MAC', 1),
('chatgpt.com', 'MAC', 1),
('t3.chat', 'MAC', 1),
('Warp', 'MAC', 0),
('Code', 'MAC', 0),
('Slack', 'MAC', 0),
('Linear', 'MAC', 0),
('Spotify', 'MAC', 0),
('Discord', 'MAC', 0),
('Activity Monitor', 'MAC', 0),
('discussions.apple.com', 'MAC', 1),
('Steam Helper', 'MAC', 0),
('zoom.us', 'MAC', 0),
('Gather', 'MAC', 0),
('Finder', 'MAC', 0),
('Automator', 'MAC', 0),
('Books', 'MAC', 0),
('Brave Browser', 'MAC', 0),
('Calculator', 'MAC', 0),
('Copilot', 'MAC', 0),
('CrossOver', 'MAC', 0),
('FaceTime', 'MAC', 0),
('Firefox', 'MAC', 0),
('Raycast', 'MAC', 0),
('Postman', 'MAC', 0),
('WhatsApp', 'MAC', 0),
('Hoppscotch', 'MAC', 0),
('apple.com', 'MAC', 1),
('messages.google.com', 'MAC', 1),
('calendar.google.com', 'MAC', 1),
('mail.google.com', 'MAC', 1),
('docs.google.com', 'MAC', 1),
('reddit.com', 'MAC', 1),
('docs.rs', 'MAC', 1);

-- Add default tag
INSERT INTO tag (name, tag_type) VALUES
('creating', 'default'),
('consuming', 'default'),
('idle', 'default');

-- Link app with tag (SQLite version)
INSERT INTO app_tag (app_id, tag_id, weight)
SELECT app.id, tag.id, 1.0
FROM app, tag
WHERE tag.name = 'creating'
AND app.name IN (
    'Cursor', 'WebStorm', 'excalidraw.com', 'stackoverflow.com', 
    'dash.cloudflare.com', 'claude.ai', 'figma.com', 'chatgpt.com', 
    't3.chat', 'Warp', 'Code', 'Calculator', 'Postman', 'docs.google.com'
);

INSERT INTO app_tag (app_id, tag_id, weight)
SELECT app.id, tag.id, 1.0
FROM app, tag
WHERE tag.name = 'consuming'
AND app.name IN (
    'github.com', '0cred.com', 'google.com', 'Arc', 'espn.com', 
    'x.com', 'youtube.com', 'facebook.com', 'instagram.com', 'Slack', 
    'Linear', 'Spotify', 'Discord', 'Activity Monitor', 'discussions.apple.com', 
    'Steam Helper', 'zoom.us', 'Gather', 'Finder', 'Automator', 'Books', 
    'Brave Browser', 'Copilot', 'CrossOver', 'FaceTime', 'Firefox', 
    'Raycast', 'WhatsApp', 'Hoppscotch', 'apple.com', 'messages.google.com', 
    'calendar.google.com', 'mail.google.com', 'reddit.com', 'docs.rs'
);