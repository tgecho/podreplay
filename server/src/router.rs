use crate::db::Db;
use crate::fetch::HttpClient;
use crate::replay;
use crate::summary;
use axum::routing::get_service;
use axum::{routing::get, AddExtensionLayer, Router};
use hyper::StatusCode;
use tower_http::{services::ServeDir, trace::TraceLayer};

pub fn make_router(db: Db, http: HttpClient) -> Router {
    Router::new()
        .fallback(get_service(ServeDir::new("ui/build")).handle_error(
            |error: std::io::Error| async move {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Unhandled internal error: {}", error),
                )
            },
        ))
        .route("/summary", get(summary::get))
        .route("/replay", get(replay::get))
        .layer(AddExtensionLayer::new(db))
        .layer(AddExtensionLayer::new(http))
        .layer(TraceLayer::new_for_http())
}
