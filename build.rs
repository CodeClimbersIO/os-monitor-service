fn main() {
    let profile = std::env::var("PROFILE").unwrap_or_else(|_| "debug".to_string());
    let should_check_db = profile != "release";

    if !should_check_db {
        println!("cargo:rustc-env=SQLX_OFFLINE=true");
    }
}
