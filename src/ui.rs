//! Embedded web UI static files.

use axum::{
    http::{header, StatusCode},
    response::{Html, IntoResponse, Response},
    routing::get,
    Router,
};

// Embed static files at compile time
const INDEX_HTML: &str = include_str!("../ui/index.html");
const AUDIT_HTML: &str = include_str!("../ui/audit.html");
const RULES_HTML: &str = include_str!("../ui/rules.html");
const STYLE_CSS: &str = include_str!("../ui/css/style.css");

pub fn ui_routes() -> Router {
    Router::new()
        .route("/", get(index))
        .route("/audit", get(audit))
        .route("/rules", get(rules))
        .route("/css/style.css", get(style_css))
}

async fn index() -> Html<&'static str> {
    Html(INDEX_HTML)
}

async fn audit() -> Html<&'static str> {
    Html(AUDIT_HTML)
}

async fn rules() -> Html<&'static str> {
    Html(RULES_HTML)
}

async fn style_css() -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/css")],
        STYLE_CSS,
    )
        .into_response()
}
