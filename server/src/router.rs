use crate::config::Config;
use crate::db::Db;
use crate::fetch::HttpClient;
use crate::replay;
use crate::summary;
use axum::routing::get_service;
use axum::{routing::get, Extension, Router};
use hyper::StatusCode;
use tower_http::services::ServeFile;
use tower_http::{services::ServeDir, trace::TraceLayer};

pub fn make_router(db: Db, http: HttpClient, config: &Config) -> Router {
    Router::new()
        .fallback_service(
            get_service(ServeDir::new(&config.assets_path))
                .handle_error(|_| async move { StatusCode::NOT_FOUND }),
        )
        .route_service(
            "/preview",
            get_service(ServeFile::new(format!(
                "{}/preview.html",
                &config.assets_path
            )))
            .handle_error(|_| async move { StatusCode::NOT_FOUND }),
        )
        .route("/summary", get(summary::get))
        .route("/replay", get(replay::get))
        .layer(Extension(db))
        .layer(Extension(http))
        .layer(TraceLayer::new_for_http())
}
