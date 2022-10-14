use crate::config::Config;
use crate::db::Db;
use crate::fetch::HttpClient;
use crate::proxy::{proxy_to, ProxyClient};
use crate::replay;
use crate::summary;
use axum::routing::{get_service, post};
use axum::{routing::get, Extension, Router};
use hyper::StatusCode;
use tower_http::{services::ServeDir, trace::TraceLayer};

pub fn make_router(db: Db, http: HttpClient, proxy: ProxyClient, config: &Config) -> Router {
    Router::new()
        .fallback(
            get_service(ServeDir::new(&config.assets_path)).handle_error(
                |error: std::io::Error| async move {
                    // TODO: Get 404s returning when appropriate
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {error}"),
                    )
                },
            ),
        )
        .route("/summary", get(summary::get))
        .route("/replay", get(replay::get))
        .route(
            "/plsbl.js",
            get(proxy_to(
                "https://plausible.io/js/plausible.js"
                    .parse()
                    .expect("Invalid URI"),
            )),
        )
        .route(
            "/api/event",
            post(proxy_to(
                "https://plausible.io/api/event"
                    .parse()
                    .expect("Invalid URI"),
            )),
        )
        .layer(Extension(db))
        .layer(Extension(http))
        .layer(Extension(proxy))
        .layer(TraceLayer::new_for_http())
}
