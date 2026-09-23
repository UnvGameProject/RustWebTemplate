mod app;
mod web;

#[tokio::main]
async fn main() {
    app::run().await;
}