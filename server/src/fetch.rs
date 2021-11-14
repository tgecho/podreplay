#![allow(clippy::large_enum_variant)]

use hyper::StatusCode;
use podreplay_lib::{Feed, ParseFeedError};
use thiserror::Error;

#[derive(Clone)]
pub struct HttpClient {
    client: reqwest::Client,
}

pub enum FetchResponse {
    NotModified,
    Fetched(Feed, Option<String>),
}

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("failed to fetch feed")]
    FetchFeedFailed(#[from] reqwest::Error),
    #[error("failed to parse feed")]
    ParseFeedFailed(#[from] ParseFeedError),
}

impl HttpClient {
    pub fn new() -> reqwest::Result<Self> {
        let client = reqwest::Client::builder().build()?;
        Ok(HttpClient { client })
    }

    pub async fn get_feed(
        &self,
        uri: &str,
        etag: Option<String>,
    ) -> Result<FetchResponse, FetchError> {
        let req = self.client.get(uri).header("User-Agent", "podreplay/0.1");
        let req = if let Some(etag) = etag {
            req.header("If-None-Match", etag)
        } else {
            req
        };
        let resp = req.send().await?;

        if resp.status() == StatusCode::NOT_MODIFIED {
            return Ok(FetchResponse::NotModified);
        }

        let etag = resp
            .headers()
            .get("etag")
            .and_then(|etag| etag.to_str().ok())
            .map(|etag| etag.to_string());

        let body = resp.bytes().await?;
        let feed = Feed::from_source(&body, Some(uri))?;

        Ok(FetchResponse::Fetched(feed, etag))
    }
}
