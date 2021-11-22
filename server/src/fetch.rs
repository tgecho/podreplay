#![allow(clippy::large_enum_variant)]

use hyper::StatusCode;
use podreplay_lib::{Feed, ParseFeedError};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::TracingMiddleware;
use thiserror::Error;

#[derive(Clone)]
pub struct HttpClient {
    client: ClientWithMiddleware,
}

impl std::fmt::Debug for HttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpClient").finish()
    }
}

pub enum FetchResponse {
    NotModified,
    Fetched(Feed, Option<String>),
}

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("failed to fetch feed")]
    Request(#[from] reqwest_middleware::Error),
    #[error("failed to read feed body")]
    Read(#[from] reqwest::Error),
    #[error("failed to parse feed")]
    Parse(#[from] ParseFeedError),
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

impl HttpClient {
    pub fn new() -> Self {
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware)
            .build();
        HttpClient { client }
    }

    #[tracing::instrument(level = "debug")]
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

        tracing::trace!("status {:?}", resp.status());

        if resp.status() == StatusCode::NOT_MODIFIED {
            return Ok(FetchResponse::NotModified);
        }

        let etag = resp
            .headers()
            .get("etag")
            .and_then(|etag| etag.to_str().ok())
            .map(|etag| etag.to_string());

        let body = resp.bytes().await?;
        tracing::trace!(?etag, ?body);

        let feed = Feed::from_source(&body, Some(uri))?;

        Ok(FetchResponse::Fetched(feed, etag))
    }
}
