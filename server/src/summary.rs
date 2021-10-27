use axum::{
    extract::{Extension, Query},
    Json,
};
use podreplay_lib::{Feed, FeedSummary};
use serde::Deserialize;
use sqlx::SqlitePool;

#[derive(Deserialize)]
pub struct SummaryQuery {
    uri: String,
}

#[derive(Debug)]
struct DbFeed {
    id: i64,
    uri: String,
}

pub async fn get(
    query: Query<SummaryQuery>,
    Extension(db): Extension<SqlitePool>,
) -> Json<FeedSummary> {
    let result = sqlx::query_as!(DbFeed, "SELECT id, uri FROM feeds;")
        .fetch_one(&db)
        .await;
    dbg!(result);

    let source = include_bytes!("serial.xml");
    let feed = Feed::from_source(source, Some("https://feeds.simplecast.com/xl36XBC2")).unwrap();
    let summary: FeedSummary = feed.into();
    Json(summary)
}
