#[tokio::main]
async fn main() {
    if let Err(error) = topcoat_poc::run().await {
        eprintln!("application startup failed: {error}");
        std::process::exit(1);
    }
}
