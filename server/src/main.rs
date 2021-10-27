mod db;
mod replay;
mod summary;

use axum::{handler::get, AddExtensionLayer, Router, Server};
use sqlx::SqlitePool;

#[tokio::main]
async fn main() {
    let pool = SqlitePool::connect("sqlite://test.sqlite").await.unwrap();
    // let result = sqlx::query_as::<_, (i32,)>("SELECT 1;")
    //     .fetch_one(&pool)
    //     .await
    //     .unwrap();
    // dbg!(result);

    let app = Router::new()
        .route("/summary", get(summary::get))
        .route("/replay", get(replay::get))
        .layer(AddExtensionLayer::new(pool));

    Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
