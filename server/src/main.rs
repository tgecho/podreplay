mod replay;
mod summary;

use axum::{handler::get, Router, Server};

#[tokio::main]
async fn main() {
    let app = Router::new()
        .route("/summary", get(summary::get))
        .route("/replay", get(replay::get));

    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
