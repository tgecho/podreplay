#![allow(clippy::large_enum_variant)]

use std::time::Duration;

use axum::body::Bytes;
use hyper::{header, StatusCode};
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::TracingMiddleware;
use thiserror::Error;
use url::Url;

use crate::helpers::HeaderMapUtils;

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

pub struct Fetched {
    pub body: Bytes,
    pub content_type: Option<String>,
    pub etag: Option<String>,
    pub url: Url,
}

#[derive(Error, Debug)]
pub enum FetchException {
    #[error("{0}")]
    Request(#[from] reqwest_middleware::Error),
    #[error("Failed to fetch feed")]
    Response(reqwest::Response),
    #[error("{0}")]
    Read(#[from] reqwest::Error),
    #[error("{0}")]
    JsonParse(#[from] serde_json::Error),
    #[error("{0}")]
    UrlParse(#[from] url::ParseError),
    #[error("Unknown")]
    Unknown,
    #[error("Unknown")]
    NotModified(Option<String>),
}

impl HttpClient {
    pub fn new(user_agent: String) -> Self {
        let client = reqwest::ClientBuilder::new()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to construct http client");
        let client = ClientBuilder::new(client)
            .with(TracingMiddleware::default())
            .build();
        HttpClient { client, user_agent }
    }

    #[tracing::instrument(level = "debug")]
    pub async fn get(&self, uri: &str, etag: Option<String>) -> Result<Fetched, FetchException> {
        let req = self
            .client
            .get(uri)
            .header(header::USER_AGENT, &self.user_agent);
        let req = if let Some(etag) = &etag {
            req.header(header::IF_NONE_MATCH, etag)
        } else {
            req
        };
        let resp = req.send().await?;

        tracing::trace!("status {:?}", resp.status());

        if resp.status() == StatusCode::NOT_MODIFIED {
            return Err(FetchException::NotModified(
                etag.or_else(|| resp.headers().get_string(header::ETAG)),
            ));
        }
        if !resp.status().is_success() {
            return Err(FetchException::Response(resp));
        }

        let url = resp.url().clone();
        let headers = resp.headers();
        let etag = headers.get_string(header::ETAG);
        let content_type = headers.get_string(header::CONTENT_TYPE);

        let body = resp.bytes().await?;
        tracing::trace!(?etag, ?body);

        Ok(Fetched {
            body,
            etag,
            content_type,
            url,
        })
    }
}
