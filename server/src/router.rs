use crate::db::Db;
use crate::fetch::HttpClient;
use crate::replay;
// use crate::summary;
use axum::{routing::get, AddExtensionLayer, Router};
use tower_http::trace::TraceLayer;

pub fn make_router(db: Db, http: HttpClient) -> Router {
    Router::new()
        // .route("/summary", get(summary::get))
        .route("/replay", get(replay::get))
        .layer(AddExtensionLayer::new(db))
        .layer(AddExtensionLayer::new(http))
        .layer(TraceLayer::new_for_http())
}
