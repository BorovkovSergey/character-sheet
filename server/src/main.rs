mod storage;
mod websocket;

use axum::{routing::get, Router};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::{
    cors::{Any, CorsLayer},
    services::ServeDir,
};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use storage::CharacterStore;
use websocket::ws_handler;

/// Shared application state available to all handlers.
#[derive(Clone)]
pub struct AppState {
    pub store: CharacterStore,
    /// When set, clients must authenticate with this password to perform mutations.
    /// When `None`, all clients have write access (backwards compatible).
    pub admin_password: Option<Arc<str>>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "server=debug,tower_http=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Read admin password from environment
    let admin_password = std::env::var("ADMIN_PASSWORD").ok().and_then(|p| {
        if p.is_empty() {
            None
        } else {
            Some(Arc::from(p.as_str()))
        }
    });
    if admin_password.is_some() {
        info!("ADMIN_PASSWORD is set — write access requires authentication");
    } else {
        info!("ADMIN_PASSWORD is not set — all clients have write access");
    }

    // Initialize storage
    let store = CharacterStore::new("data").await;

    let state = AppState {
        store,
        admin_password,
    };

    // CORS layer for development
    // TODO: Restrict CORS origins in production to specific allowed domains
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // Build router
    let app = Router::new()
        .route("/ws", get(ws_handler))
        .nest_service("/", ServeDir::new("static"))
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind TCP listener - port 8080 may be in use");
    axum::serve(listener, app)
        .await
        .expect("Server terminated unexpectedly");
}
