#[derive(Debug, sqlx::Type, PartialEq, Clone)]
#[sqlx(type_name = "TEXT", rename_all = "UPPERCASE")]
pub enum Platform {
    Mac,
    Windows,
    Linux,
    Ios,
    Android,
    Unknown,
    Web,
}

impl From<String> for Platform {
    fn from(s: String) -> Self {
        match s.as_str() {
            "MAC" => Platform::Mac,
            "WINDOWS" => Platform::Windows,
            "LINUX" => Platform::Linux,
            "IOS" => Platform::Ios,
            "ANDROID" => Platform::Android,
            _ => Platform::Unknown,
        }
    }
}

impl From<os_monitor::Platform> for Platform {
    fn from(platform: os_monitor::Platform) -> Self {
        match platform {
            os_monitor::Platform::Mac => Platform::Mac,
            os_monitor::Platform::Windows => Platform::Windows,
            os_monitor::Platform::Linux => Platform::Linux,
        }
    }
}
