use os_monitor::WindowEvent;
use sqlx::Row;
use time::OffsetDateTime;
use url;

use crate::db::types::Platform;

#[derive(Clone, Debug)]
pub struct App {
    pub id: Option<i64>,
    pub name: String,
    pub platform: Platform,
    pub is_browser: bool,
    pub created_at: Option<OffsetDateTime>,
    pub updated_at: Option<OffsetDateTime>,
}

impl<'r> sqlx::FromRow<'r, sqlx::sqlite::SqliteRow> for App {
    fn from_row(row: &'r sqlx::sqlite::SqliteRow) -> Result<Self, sqlx::Error> {
        Ok(App {
            id: row.try_get("id")?,
            name: row.try_get("name")?,
            platform: row.try_get("platform")?,
            is_browser: row.try_get("is_browser")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        })
    }
}

impl App {
    pub fn new(event: &WindowEvent) -> App {
        App {
            id: None,
            created_at: None,
            updated_at: None,
            name: match &event.url {
                Some(url) => Self::get_domain_from_url(url),
                None => event.app_name.clone(),
            },
            platform: event.platform.clone().into(),
            is_browser: event.url.is_some(),
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
            id: None,
            created_at: None,
            updated_at: None,
            name: "Test App".to_string(),
            platform: Platform::Mac,
            is_browser: false,
        }
    }

    #[cfg(test)]
    pub fn __create_test_apps(names: &Vec<String>) -> Vec<App> {
        names
            .iter()
            .map(|name| App {
                id: None,
                created_at: None,
                updated_at: None,
                name: name.to_string(),
                platform: Platform::Mac,
                is_browser: false,
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
