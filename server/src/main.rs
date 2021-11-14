mod db;
mod fetch;
mod replay;
mod summary;

use axum::{routing::get, AddExtensionLayer, Router, Server};
use db::Db;

use crate::fetch::HttpClient;

#[tokio::main]
async fn main() {
    let db = Db::new("sqlite://test.sqlite")
        .await
        .expect("Failed to open sqlite");
    println!("SQLite version {}", db.get_version().await.unwrap());

    let http = HttpClient::new().expect("Failed to creat HTTP client");

    let app = Router::new()
        .route("/summary", get(summary::get))
        .route("/replay", get(replay::get))
        .layer(AddExtensionLayer::new(db))
        .layer(AddExtensionLayer::new(http));

    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
