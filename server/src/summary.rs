use podreplay_lib::{Feed, FeedSummary};
use serde::Deserialize;
use warp::reply::{json, Json};

#[derive(Deserialize)]
pub struct SummaryQuery {
    uri: String,
}

pub fn get(query: SummaryQuery) -> Json {
    dbg!(query.uri);
    let source = include_bytes!("serial.xml");
    let feed = Feed::from_source(source, Some("https://feeds.simplecast.com/xl36XBC2"));
    let summary: FeedSummary = feed.into();
    json(&summary)
}
