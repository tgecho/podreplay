use std::io::{BufRead, Cursor, Seek};

use crate::fetch::{FetchError, FetchResponse, HttpClient};
use axum::{
    body::{boxed, BoxBody},
    extract::{Extension, Query},
    response::{IntoResponse, Response},
    Json,
};
use headers::{HeaderMap, HeaderValue};
use hyper::{body::Buf, Body, StatusCode};
use podreplay_lib::{find_feed_links, FeedSummary, SummarizeError};
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
) -> Result<SummaryResponse, SummaryError> {
    // TODO: try to add http(s) if missing?

    let if_none_match = headers
        .get("if-none-match")
        .and_then(|inm| inm.to_str().ok())
        .map(|s| s.to_string());
    tracing::debug!("If-None-Match: {:?}", if_none_match);
    let fetched = http.get_feed(&query.uri, if_none_match).await?;

    let (feed_body, fetched_etag) = match fetched {
        FetchResponse::NotModified => {
            tracing::debug!("NotModified (feed returned 304)");
            return Ok(SummaryResponse::NotModified);
        }
        FetchResponse::Fetched { body, etag, .. } => (body, etag),
    };

    let mut reader = Cursor::new(feed_body);
    let summary = match FeedSummary::new(query.uri.clone(), &mut reader) {
        Ok(summary) => summary,
        Err(_) => {
            reader.rewind()?;
            attempt_autodiscovery(&mut reader, &query.uri, http).await?
        }
    };
    let mut headers = HeaderMap::new();
    if let Some(etag) = fetched_etag.and_then(|etag| HeaderValue::from_str(etag.as_ref()).ok()) {
        headers.append("ETag", etag);
    }
    Ok(SummaryResponse::Success { headers, summary })
}

async fn attempt_autodiscovery<R: BufRead>(
    reader: &mut R,
    origin: &str,
    http: HttpClient,
) -> Result<FeedSummary, SummarizeError> {
    for uri in find_feed_links(reader, origin) {
        if let Ok(FetchResponse::Fetched { body, .. }) = http.get_feed(&uri, None).await {
            let mut reader = body.reader();
            let summary = FeedSummary::new(uri, &mut reader);
            if summary.is_ok() {
                return summary;
            }
        }
    }
    Err(SummarizeError::NotAFeed)
}

pub enum SummaryResponse {
    NotModified,
    Success {
        headers: HeaderMap,
        summary: FeedSummary,
    },
}

impl IntoResponse for SummaryResponse {
    fn into_response(self) -> Response<BoxBody> {
        match self {
            SummaryResponse::NotModified => StatusCode::NOT_MODIFIED.into_response(),
            SummaryResponse::Success { headers, summary } => {
                (headers, Json(summary)).into_response()
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum SummaryError {
    #[error("{0}")]
    FetchError(#[from] FetchError),
    #[error("{0}")]
    ParseError(#[from] SummarizeError),
    #[error("Unexpected internal error")]
    UnknownError(#[from] std::io::Error),
}

impl IntoResponse for SummaryError {
    fn into_response(self) -> Response<BoxBody> {
        tracing::error!(?self);
        let body = Body::from(self.to_string());
        let status = match self {
            Self::UnknownError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::BAD_GATEWAY,
        };
        Response::builder()
            .status(status)
            .body(boxed(body))
            .expect("Failed to build error response")
    }
}
