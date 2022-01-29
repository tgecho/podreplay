#![allow(clippy::large_enum_variant)]

use std::net::IpAddr;

use axum::body::Bytes;
use headers::{HeaderName, HeaderValue};
use hyper::{header, Body, Request, StatusCode};
use lazy_static::lazy_static;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware};
use reqwest_tracing::TracingMiddleware;
use serde_json::json;
use thiserror::Error;

#[derive(Clone)]
pub struct HttpClient {
    user_agent: String,
    client: ClientWithMiddleware,
    analytics_target: Option<String>,
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
    pub fn new(user_agent: String, analytics_target: Option<String>) -> Self {
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware)
            .build();
        HttpClient {
            client,
            user_agent,
            analytics_target,
        }
    }

    #[tracing::instrument(level = "debug")]
    pub async fn get_feed(
        &self,
        uri: &str,
        etag: Option<String>,
    ) -> Result<FetchResponse, FetchError> {
        let req = self
            .client
            .get(uri)
            .header(header::USER_AGENT, &self.user_agent);
        let req = if let Some(etag) = etag {
            req.header(header::IF_NONE_MATCH, etag)
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
            .get(header::ETAG)
            .and_then(|etag| etag.to_str().ok())
            .map(|etag| etag.to_string());
        let content_type = headers
            .get(header::CONTENT_TYPE)
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

    pub fn record_event(&self, name: &str, client_addr: &IpAddr, req: &Request<Body>) {
        if let Some(analytics_target) = &self.analytics_target {
            lazy_static! {
                static ref NOT_SENT: HeaderValue = "not sent".parse().unwrap();
            }

            let user_agent = req.headers().get(header::USER_AGENT).unwrap_or(&NOT_SENT);
            let report = self
                .client
                .get(analytics_target)
                .header(
                    "X-Forwarded-For".parse::<HeaderName>().unwrap(),
                    client_addr.to_string(),
                )
                .header(header::USER_AGENT, user_agent)
                .header(header::CONTENT_TYPE, "application/json")
                .body(
                    json!({
                        "name": name,
                        "url": req.uri().to_string(),
                    })
                    .to_string(),
                );
            tokio::spawn(report.send());
        }
    }
}
