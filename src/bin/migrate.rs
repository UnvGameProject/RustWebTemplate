#[tokio::main]
async fn main() {
    if let Err(error) = topcoat_poc::run_migrations().await {
        eprintln!("database migration failed: {error}");
        std::process::exit(1);
    }
}
