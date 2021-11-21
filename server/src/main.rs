mod db;
mod fetch;
pub mod replay;
mod summary;

use axum::{routing::get, AddExtensionLayer, Router, Server};
use db::Db;
use tower_http::trace::TraceLayer;

use crate::fetch::HttpClient;

pub fn make_app(db: Db, http: HttpClient) -> Router {
    Router::new()
        .route("/summary", get(summary::get))
        .route("/replay", get(replay::get))
        .layer(AddExtensionLayer::new(db))
        .layer(AddExtensionLayer::new(http))
        .layer(TraceLayer::new_for_http())
}

#[tokio::main]
async fn main() {
    color_eyre::install().unwrap();

    #[cfg(debug_assertions)]
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "debug");
    }
    tracing_subscriber::fmt::init();

    let db = Db::new("sqlite://test.sqlite")
        .await
        .expect("Failed to open sqlite");

    let http = HttpClient::new();

    let app = make_app(db, http);

    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
