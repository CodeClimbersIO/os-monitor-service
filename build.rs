fn main() {
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let should_check_db = profile != "release";

    if !should_check_db {
        println!("Running SQLx database operations...");
        println!("cargo:rustc-env=SQLX_OFFLINE=true");
    } else {
        // Load .env file first
        // log current directory
        println!(
            "cargo:warning=Current directory: {}",
            std::env::current_dir().unwrap().display()
        );
        if let Err(e) = dotenv::dotenv() {
            println!("cargo:warning=Failed to load .env file: {}", e);
        }
        // Run migrations
        let migrate_status = std::process::Command::new("sqlx")
            .args(&["migrate", "run"])
            .status()
            .expect("Failed to execute SQLx migrate command");

        if !migrate_status.success() {
            panic!("SQLx migrations failed to run");
        }
    }
}
