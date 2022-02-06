use crate::{
    autodiscovery::{AutodiscoveryException, FeedUrl},
    fetch::{FetchException, HttpClient},
    helpers::HeaderMapUtils,
};
use axum::{
    body::BoxBody,
    extract::{Extension, Query},
    response::{IntoResponse, Response},
    Json,
};
use headers::HeaderMap;
use hyper::{header, StatusCode};
use podreplay_lib::{FeedSummary, SummarizeError};
use reqwest::Url;
use serde::Deserialize;
use thiserror::Error;

#[derive(Deserialize)]
pub struct SummaryQuery {
    uri: String,
}

pub async fn get(
    query: Query<SummaryQuery>,
    headers: HeaderMap,
    Extension(http): Extension<HttpClient>,
) -> Result<Summary, SummaryError> {
    // TODO: maybe return something that indicates how we found it?

    let url = Url::parse(&query.uri)
        .or_else(|original| Url::parse(&format!("http://{}", &query.uri)).map_err(|_| original))?;

    let feed_url = FeedUrl::new(url);

    let if_none_match = headers.get_string(header::IF_NONE_MATCH);
    tracing::debug!("If-None-Match: {:?}", if_none_match);

    let found = feed_url.attempt_autodiscovery(&http, if_none_match).await?;

    let mut headers = HeaderMap::new();
    headers.try_append(header::ETAG, found.etag);
    Ok(Summary {
        headers,
        summary: found.summary,
    })
}

pub struct Summary {
    headers: HeaderMap,
    summary: FeedSummary,
}

impl IntoResponse for Summary {
    fn into_response(self) -> Response<BoxBody> {
        (self.headers, Json(self.summary)).into_response()
    }
}

#[derive(Error, Debug)]
pub enum SummaryError {
    #[error("{0}")]
    Fetch(#[from] FetchException),
    #[error("{0}")]
    Parse(#[from] SummarizeError),
    #[error("Unexpected internal error")]
    Io(#[from] std::io::Error),
    #[error("Autodiscovery failed")]
    Autodiscovery(#[from] AutodiscoveryException),
    #[error("Invalid url")]
    UrlParse(#[from] url::ParseError),
    #[error("Unexpected internal error")]
    Unknown,
}

impl IntoResponse for SummaryError {
    fn into_response(self) -> Response<BoxBody> {
        match self {
            Self::Autodiscovery(AutodiscoveryException::Fetch(FetchException::NotModified(
                etag,
            ))) => {
                let mut headers = HeaderMap::new();
                headers.try_append(header::ETAG, etag);
                (headers, StatusCode::NOT_MODIFIED).into_response()
            }
            Self::Autodiscovery(AutodiscoveryException::Failed) => {
                (StatusCode::NOT_FOUND, "Unable to find a feed").into_response()
            }
            Self::Unknown | Self::Io(_) => {
                tracing::error!(?self);
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            }
            _ => {
                tracing::error!(?self);
                StatusCode::BAD_GATEWAY.into_response()
            }
        }
    }
}
