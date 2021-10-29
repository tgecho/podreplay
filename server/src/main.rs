mod db;
mod replay;
mod summary;

use axum::{handler::get, AddExtensionLayer, Router, Server};
use db::Db;

#[tokio::main]
async fn main() {
    let db = Db::new("sqlite://test.sqlite").await.unwrap();
    println!("SQLite version {}", db.get_version().await.unwrap());

    let app = Router::new()
        .route("/summary", get(summary::get))
        .route("/replay", get(replay::get))
        .layer(AddExtensionLayer::new(db));

    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
