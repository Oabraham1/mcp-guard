//! HTTP API for mcp-guard.

pub mod routes;
pub mod state;

use crate::db::DbPool;
use crate::ui;
use axum::{
    routing::{delete, get, post, put},
    Router,
};
use state::AppState;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn create_router(db: DbPool) -> Router {
    let state = AppState::new(db);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    // API routes that need state
    let api_routes = Router::new()
        // Audit
        .route("/api/audit", get(routes::audit::list_audit))
        // Rules
        .route("/api/rules", get(routes::rules::list_rules))
        .route("/api/rules", post(routes::rules::create_rule))
        .route("/api/rules/:id", put(routes::rules::update_rule))
        .route("/api/rules/:id", delete(routes::rules::delete_rule))
        .with_state(state);

    // Stateless routes
    let stateless_routes = Router::new()
        // UI routes
        .merge(ui::ui_routes())
        // Health
        .route("/api/health", get(routes::health::health))
        // Servers
        .route("/api/servers", get(routes::servers::list_servers))
        // Scan
        .route("/api/scan", post(routes::scan::run_scan));

    stateless_routes
        .merge(api_routes)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}

pub async fn serve(db: DbPool, bind: &str, port: u16) -> crate::error::Result<()> {
    let app = create_router(db);
    let listener = tokio::net::TcpListener::bind(format!("{}:{}", bind, port)).await?;

    tracing::info!("API server listening on {}:{}", bind, port);

    axum::serve(listener, app)
        .await
        .map_err(|e| crate::error::Error::Other(e.to_string()))?;

    Ok(())
}
