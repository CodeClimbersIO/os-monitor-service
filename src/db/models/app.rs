use os_monitor::WindowEvent;
use sqlx::Row;
use time::OffsetDateTime;
use url;
use uuid;

use crate::db::types::Platform;

#[derive(Clone, Debug)]
pub struct App {
    pub id: Option<String>,
    pub name: Option<String>,
    pub app_external_id: String,
    pub platform: Platform,
    pub is_browser: bool,
    pub is_default: bool,
    pub is_blocked: bool,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for App {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(App {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            app_external_id: row.try_get("app_external_id")?,
            platform: row.try_get("platform")?,
            is_browser: row.try_get("is_browser")?,
            is_default: row.try_get("is_default")?,
            is_blocked: row.try_get("is_blocked")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl App {
    pub fn new(event: &WindowEvent) -> App {
        // if the event has a url, we use the url as the app_external_id
        // if the event has a bundle_id, we use the bundle_id as the app_external_id
        // if the event has neither, we use the app_name as the app_external_id
        let url = match &event.url {
            Some(url) => Some(Self::get_domain_from_url(url)),
            None => None,
        };

        let app_external_id = if let Some(url) = url {
            url.clone()
        } else if let Some(bundle_id) = &event.bundle_id {
            bundle_id.clone()
        } else {
            event.app_name.clone()
        };

        App {
            id: Some(uuid::Uuid::new_v4().to_string()),
            created_at: None,
            updated_at: None,
            name: Some(event.app_name.clone()),
            app_external_id: app_external_id,
            platform: event.platform.clone().into(),
            is_browser: event.url.is_some(),
            is_default: false,
            is_blocked: false,
        }
    }

    pub fn get_domain_from_url(url: &str) -> String {
        if let Ok(parsed) = url::Url::parse(url) {
            if let Some(domain) = parsed.host_str() {
                // Remove 'www.' prefix if present
                return domain.strip_prefix("www.").unwrap_or(domain).to_string();
            }
        }

        let domain = url
            .trim_start_matches("www.")
            .trim_end_matches('/')
            .split('/')
            .next()
            .unwrap_or(url)
            .split('?')
            .next()
            .unwrap_or(url);

        domain.to_string()
    }

    #[cfg(test)]
    pub fn __create_test_app() -> App {
        App {
            id: Some(uuid::Uuid::new_v4().to_string()),
            created_at: None,
            updated_at: None,
            name: Some("Test App".to_string()),
            app_external_id: "".to_string(),
            platform: Platform::Mac,
            is_browser: false,
            is_default: false,
            is_blocked: false,
        }
    }

    #[cfg(test)]
    pub fn __create_test_apps(names: &Vec<String>) -> Vec<App> {
        names
            .iter()
            .map(|name| App {
                id: Some(uuid::Uuid::new_v4().to_string()),
                created_at: None,
                updated_at: None,
                name: Some(name.to_string()),
                app_external_id: "".to_string(),
                platform: Platform::Mac,
                is_browser: false,
                is_default: false,
                is_blocked: false,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_domain_from_url() {
        let test_cases = vec![
            ("https://www.google.com/", "google.com"),
            ("https://google.com/route?withParam", "google.com"),
            ("www.google.com", "google.com"),
            ("google.com", "google.com"),
            ("https://sub.domain.com/path", "sub.domain.com"),
            ("https://www.sub.domain.com/path", "sub.domain.com"),
            ("http://localhost:3000", "localhost"),
            ("example.org/path/to/resource", "example.org"),
        ];

        for (input, expected) in test_cases {
            assert_eq!(
                App::get_domain_from_url(input),
                expected,
                "Failed for input: {}",
                input
            );
        }
    }
}
