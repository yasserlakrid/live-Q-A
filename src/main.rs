mod handlers;
mod models;
mod routes;
mod state;
mod websocket;

use std::net::SocketAddr;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new("info"))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let app = routes::create_app();
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));

    tracing::info!("Starting ESI welcome-day survey server on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind TCP listener");

    axum::serve(listener, app)
        .await
        .expect("server failed to run");
}
