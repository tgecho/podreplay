use axum::{extract::Query, Json};
use podreplay_lib::{Feed, FeedSummary};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct SummaryQuery {
    uri: String,
}

pub async fn get(query: Query<SummaryQuery>) -> Json<FeedSummary> {
    let source = include_bytes!("serial.xml");
    let _ = query.uri;
    let feed = Feed::from_source(source, Some("https://feeds.simplecast.com/xl36XBC2")).unwrap();
    let summary: FeedSummary = feed.into();
    Json(summary)
}
