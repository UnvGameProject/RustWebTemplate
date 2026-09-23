mod app;
mod config;
mod db;
mod web;

#[tokio::main]
async fn main() {
    if let Err(error) = app::run().await {
        eprintln!("application startup failed: {error}");
        std::process::exit(1);
    }
}