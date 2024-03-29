#![allow(clippy::large_enum_variant)]

use std::net::SocketAddr;

use axum::{
    body::{Body, BoxBody},
    extract::{ConnectInfo, Extension, Query},
    response::IntoResponse,
};
use chrono::{DateTime, SecondsFormat, Utc};
use headers::{HeaderMap, HeaderValue};
use hyper::{Request, Response, StatusCode};
use lazy_static::lazy_static;
use podreplay_lib::{
    create_cached_entry_map, diff_feed, parse_rule, parse_timestamp, reschedule_feed, rewrite_feed,
    FeedSummary, RewriteError, SummarizeError,
};
use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

use crate::{
    db::Db,
    fetch::{FetchException, HttpClient},
    helpers::HeaderMapUtils,
};

#[derive(Deserialize, Debug)]
pub struct ReplayQuery {
    rule: String,
    start: String,
    first: Option<DateTime<Utc>>,
    last: Option<DateTime<Utc>>,
    uri: String,
    title: Option<String>,
    now: Option<DateTime<Utc>>,
}

#[tracing::instrument]
pub async fn get<'a>(
    query: Query<ReplayQuery>,
    Extension(db): Extension<Db>,
    Extension(http): Extension<HttpClient>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    request: Request<Body>,
) -> Result<Replay, ReplayError> {
    let clock_now = Utc::now();
    let now = query.now.unwrap_or(clock_now);
    if (now - clock_now).num_days() > 365 {
        return Err(ReplayError::InvalidRequest(
            "Please don't specify a ?now beyond 1 year".to_string(),
        ));
    }

    let headers = request.headers();

    let if_none_match = headers.get_str("if-none-match");
    tracing::debug!("If-None-Match: {:?}", if_none_match);
    let (feed_request_etag, request_expires) =
        if_none_match.map_or((None, None), parse_request_etag);
    if let Some(expires) = request_expires {
        if now < expires {
            tracing::debug!("NotModified ({} < {:?})", now, expires);
            return Err(ReplayError::NotModified {
                headers: prepare_headers(request_expires, feed_request_etag.map(|e| e.to_string())),
            });
        } else {
            tracing::debug!("NotModified {:?}", request_expires);
        }
    }

    let fetched = http
        .get(
            &query.uri,
            feed_request_etag.map(|etag| format!(r#""{etag}""#)),
        )
        .await;

    let fetched = match fetched {
        Ok(fetched) => fetched,
        Err(FetchException::NotModified(_)) => {
            return Err(ReplayError::NotModified {
                headers: prepare_headers(request_expires, feed_request_etag.map(|e| e.to_string())),
            });
        }
        err => err?,
    };

    let summary = FeedSummary::new(query.uri.clone(), &fetched.body)?;

    let (feed_meta, entries) =
        get_updated_caches(db, &query.uri, now, &fetched.etag, &summary).await?;

    let query_start = parse_timestamp(&query.start).ok_or_else(|| {
        ReplayError::InvalidRequest(format!("Unable to parse timestamp {}", query.start))
    })?;
    let rule = parse_rule(query_start, &query.rule);

    let (replayed, next_slot) = reschedule_feed(
        &entries,
        rule,
        query_start,
        Some(now),
        feed_meta.first_fetched,
        query.first,
        query.last,
    );

    let body = rewrite_feed(
        &fetched.body,
        &replayed,
        true,
        !summary.marked_private,
        &query.title,
    )?;
    let mut headers = prepare_headers(next_slot, fetched.etag);
    headers.append(
        "Content-Type",
        HeaderValue::from_str(
            &fetched
                .content_type
                .unwrap_or_else(|| "application/rss+xml".to_string()),
        )
        .unwrap(),
    );
    Ok(Replay { body, headers })
}

async fn get_updated_caches(
    db: Db,
    uri: &str,
    now: DateTime<Utc>,
    fetched_etag: &Option<String>,
    feed: &FeedSummary,
) -> Result<(podreplay_lib::FeedMeta, Vec<podreplay_lib::CachedEntry>), ReplayError> {
    let feed_meta = db.update_feed_meta(uri, &now, fetched_etag).await?;

    let cached_entries = db.get_entries(feed_meta.id).await?;
    let cached_entry_map = create_cached_entry_map(&cached_entries);

    let changes = diff_feed(&feed.id_map(), &cached_entry_map, feed_meta.id, now);
    let entries = if changes.is_empty() {
        cached_entries
    } else {
        db.update_cached_entries(feed_meta.id, &changes).await?
    };

    Ok((feed_meta, entries))
}

fn prepare_headers(next_slot: Option<DateTime<Utc>>, fetched_etag: Option<String>) -> HeaderMap {
    // TODO: forward on any other safe/relevant feed caching related headers?
    let mut headers = HeaderMap::new();
    if let Some(expires) = next_slot.and_then(|dt| HeaderValue::from_str(&dt.to_rfc2822()).ok()) {
        headers.insert("Expires", expires);
    }
    if let Some(feed_etag) = fetched_etag.and_then(|e| Some(extract_etag_value(&e)?.to_string())) {
        let etag = if let Some(next_dt) =
            next_slot.map(|dt| dt.to_rfc3339_opts(SecondsFormat::Secs, true))
        {
            format!(r#""{next_dt}|{feed_etag}""#)
        } else {
            format!(r#""{feed_etag}""#)
        };
        if let Ok(etag) = HeaderValue::from_str(&etag) {
            headers.insert("Etag", etag);
        }
    }
    headers
}

lazy_static! {
    static ref ETAG_RE: Regex = Regex::new(r#"^(?:W/)?"?([^"]+)"?$"#).unwrap();
}

fn parse_rfc3339(dt: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    Ok(DateTime::parse_from_rfc3339(dt)?.into())
}

fn extract_etag_value(inm: &str) -> Option<&str> {
    let cap = ETAG_RE.captures(inm)?;
    cap.get(1).map(|g| g.as_str())
}

fn parse_request_etag(if_none_match: &str) -> (Option<&str>, Option<DateTime<Utc>>) {
    let split_inm = extract_etag_value(if_none_match).map(|cap| match cap.split_once('|') {
        Some((expires, etag)) => {
            let expires = parse_rfc3339(expires)
                .map_err(|err| {
                    tracing::warn!(
                        "Failed to parse timestamp in If-None-Match: {}; {}",
                        if_none_match,
                        err
                    );
                    err
                })
                .ok();
            (expires, etag)
        }
        None => (None, cap),
    });
    let feed_request_etag = split_inm.map(|inm| inm.1);
    let request_expires = split_inm.and_then(|inm| inm.0);
    (feed_request_etag, request_expires)
}

pub struct Replay {
    headers: HeaderMap,
    body: Vec<u8>,
}

impl IntoResponse for Replay {
    fn into_response(self) -> Response<BoxBody> {
        (self.headers, self.body).into_response()
    }
}

#[derive(Error, Debug)]
pub enum ReplayError {
    #[error("Invalid request: {0}")]
    InvalidRequest(String),
    #[error("{0}")]
    FetchError(#[from] FetchException),
    #[error("{0}")]
    ParseError(#[from] SummarizeError),
    #[error("{0}")]
    WriteError(#[from] RewriteError),
    #[error("Unexpected internal error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("Unexpected internal error")]
    UnknownError(#[from] std::io::Error),
    #[error("Not modified")]
    NotModified { headers: HeaderMap },
}

impl IntoResponse for ReplayError {
    fn into_response(self) -> Response<BoxBody> {
        tracing::error!(?self);
        match self {
            Self::NotModified { headers } => (headers, StatusCode::NOT_MODIFIED).into_response(),
            Self::InvalidRequest(message) => (StatusCode::BAD_REQUEST, message).into_response(),
            Self::FetchError(_) | Self::ParseError(_) => StatusCode::BAD_GATEWAY.into_response(),
            _ => StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    }
}
