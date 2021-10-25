use chrono::{DateTime, Utc};
use podreplay_lib::{Feed, FeedSummary};
use serde::Deserialize;
use warp::{
    reply::{json, Json},
    Reply,
};

#[derive(Deserialize, Debug)]
pub struct SummaryQuery {
    // start: DateTime<Utc>,
    uri: String,
}

pub async fn get(query: SummaryQuery) -> Result<impl Reply, reqwest::Error> {
    dbg!(&query);
    let feed = fetch_feed(&query.uri).await?;
    let summary: FeedSummary = feed.into();
    Ok(json(&summary))
}

async fn fetch_feed(uri: &str) -> Result<Feed, reqwest::Error> {
    let client = reqwest::Client::builder().build()?;
    let resp = client
        .get(uri)
        .header("User-Agent", "podreplay/0.1")
        .send()
        .await?;
    let body = resp.bytes().await?;
    let feed = Feed::from_source(&body, Some(uri));
    Ok(feed)
}
