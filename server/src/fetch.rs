#![allow(clippy::large_enum_variant)]

use axum::body::Bytes;
use hyper::StatusCode;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::TracingMiddleware;
use thiserror::Error;

#[derive(Clone)]
pub struct HttpClient {
    user_agent: String,
    client: ClientWithMiddleware,
}

impl std::fmt::Debug for HttpClient {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("HttpClient").finish()
    }
}

pub enum FetchResponse {
    NotModified,
    Fetched {
        body: Bytes,
        etag: Option<String>,
        content_type: Option<String>,
    },
}

#[derive(Error, Debug)]
pub enum FetchError {
    #[error("{0}")]
    Request(#[from] reqwest_middleware::Error),
    #[error("Failed to fetch feed")]
    Response(reqwest::Response),
    #[error("{0}")]
    Read(#[from] reqwest::Error),
}

impl HttpClient {
    pub fn new(user_agent: String) -> Self {
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware)
            .build();
        HttpClient { client, user_agent }
    }

    #[tracing::instrument(level = "debug")]
    pub async fn get_feed(
        &self,
        uri: &str,
        etag: Option<String>,
    ) -> Result<FetchResponse, FetchError> {
        let req = self.client.get(uri).header("User-Agent", &self.user_agent);
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
        if !resp.status().is_success() {
            return Err(FetchError::Response(resp));
        }

        let headers = resp.headers();
        let etag = headers
            .get("etag")
            .and_then(|etag| etag.to_str().ok())
            .map(|etag| etag.to_string());
        let content_type = headers
            .get("content-type")
            .and_then(|ct| ct.to_str().ok())
            .map(|ct| ct.to_string());

        let body = resp.bytes().await?;
        tracing::trace!(?etag, ?body);

        Ok(FetchResponse::Fetched {
            body,
            etag,
            content_type,
        })
    }
}
