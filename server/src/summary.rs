use podreplay_lib::{parse_feed, FeedSummary};
use serde::Deserialize;
use warp::reply::{json, Json};

#[derive(Deserialize)]
pub struct SummaryQuery {
    uri: String,
}

pub fn get(query: SummaryQuery) -> Json {
    dbg!(query.uri);
    let source = include_bytes!("serial.xml");
    let feed = parse_feed(source, Some("https://feeds.simplecast.com/xl36XBC2"));
    let summary: FeedSummary = feed.into();
    json(&summary)
}
