use axum::{body::Body, extract::Query, response::IntoResponse, Json};
use chrono::{DateTime, Utc};
use hyper::{Response, StatusCode};
use podreplay_lib::{Feed, FeedSummary, ParseFeedError};
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize, Debug)]
pub struct SummaryQuery {
    start: Option<DateTime<Utc>>,
    uri: String,
}

pub async fn get(query: Query<SummaryQuery>) -> Result<Json<FeedSummary>, ReplayError> {
    dbg!(&query);
    let feed = fetch_feed(&query.uri).await?;
    let summary: FeedSummary = feed.into();
    Ok(Json(summary))
}

async fn fetch_feed(uri: &str) -> Result<Feed, ReplayError> {
    let client = reqwest::Client::builder().build()?;
    let resp = client
        .get(uri)
        .header("User-Agent", "podreplay/0.1")
        .send()
        .await?;
    let body = resp.bytes().await?;
    let feed = Feed::from_source(&body, Some(uri))?;
    Ok(feed)
}

#[derive(Error, Debug)]
pub enum ReplayError {
    #[error("failed to fetch feed")]
    FetchFeedFailed(#[from] reqwest::Error),
    #[error("failed to parse feed")]
    ParseFeedFailed(#[from] ParseFeedError),
}

impl IntoResponse for ReplayError {
    type Body = Body;
    type BodyError = <Self::Body as axum::body::HttpBody>::Error;

    fn into_response(self) -> Response<Self::Body> {
        let body = match self {
            ReplayError::FetchFeedFailed(err) => Body::from(err.to_string()),
            ReplayError::ParseFeedFailed(err) => Body::from(err.to_string()),
        };

        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body)
            .unwrap()
    }
}
