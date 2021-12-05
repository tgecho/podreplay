#![allow(clippy::large_enum_variant)]

use axum::{
    body::{boxed, Body, BoxBody, HttpBody},
    extract::{Extension, Query},
    response::IntoResponse,
};
use chrono::{DateTime, SecondsFormat, Utc};
use chronoutil::DateRule;
use headers::{HeaderMap, HeaderValue};
use hyper::{Response, StatusCode};
use lazy_static::lazy_static;
use podreplay_lib::{diff_feed, feed::create_cached_entry_map, replay_feed, Feed, ReplayedItem};
use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

use crate::{
    db::Db,
    fetch::{FetchError, FetchResponse, HttpClient},
};

#[derive(Deserialize, Debug)]
pub struct SummaryQuery {
    start: DateTime<Utc>,
    uri: String,
    #[cfg(test)]
    now: DateTime<Utc>,
}

#[tracing::instrument]
pub async fn get<'a>(
    query: Query<SummaryQuery>,
    headers: HeaderMap,
    Extension(db): Extension<Db>,
    Extension(http): Extension<HttpClient>,
) -> Result<ReplayResponse, ReplayError> {
    #[cfg(test)]
    let now = query.now;
    #[cfg(not(test))]
    let now = Utc::now();

    let if_none_match = headers
        .get("if-none-match")
        .and_then(|inm| inm.to_str().ok());
    tracing::debug!("If-None-Match: {:?}", if_none_match);
    let (feed_request_etag, request_expires) =
        if_none_match.map_or((None, None), parse_request_etag);
    if let Some(expires) = request_expires {
        if now < expires {
            tracing::debug!("NotModified ({} < {:?})", now, expires);
            return Ok(ReplayResponse::NotModified {
                headers: prepare_headers(request_expires, feed_request_etag.map(|e| e.to_string())),
            });
        } else {
            tracing::debug!("NotModified {:?}", request_expires);
        }
    }

    let fetched = http
        .get_feed(
            &query.uri,
            feed_request_etag.map(|etag| format!(r#""{}""#, etag)),
        )
        .await?;

    let (feed, fetched_etag) = match fetched {
        FetchResponse::NotModified => {
            tracing::debug!("NotModified (feed returned 304)");
            return Ok(ReplayResponse::NotModified {
                headers: prepare_headers(request_expires, feed_request_etag.map(|e| e.to_string())),
            });
        }
        FetchResponse::Fetched(feed, fetched_etag) => (feed, fetched_etag),
    };

    tracing::trace!(?feed);

    let (feed_meta, entries) =
        get_updated_caches(db, &query.uri, now, &fetched_etag, &feed).await?;

    let rule = DateRule::weekly(query.start);

    let (replayed, next_slot) =
        replay_feed(&entries, rule, query.start, now, feed_meta.first_fetched);

    let headers = prepare_headers(next_slot, fetched_etag);

    // TODO: use replayed and the fetched feed to build a final feed
    Ok(ReplayResponse::Success {
        feed,
        schedule: replayed,
        headers,
    })
}

async fn get_updated_caches(
    db: Db,
    uri: &str,
    now: DateTime<Utc>,
    fetched_etag: &Option<String>,
    feed: &Feed,
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
            format!(r#""{}|{}""#, next_dt, feed_etag)
        } else {
            format!(r#""{}""#, feed_etag)
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

pub enum ReplayResponse {
    NotModified {
        headers: HeaderMap,
    },
    Success {
        headers: HeaderMap,
        feed: Feed,
        schedule: Vec<ReplayedItem>,
    },
}

impl IntoResponse for ReplayResponse {
    type Body = BoxBody;
    type BodyError = <Self::Body as HttpBody>::Error;

    fn into_response(self) -> Response<Self::Body> {
        match self {
            ReplayResponse::NotModified { headers } => (headers, StatusCode::NOT_MODIFIED)
                .into_response()
                .map(boxed),
            ReplayResponse::Success {
                mut headers,
                feed,
                schedule,
            } => {
                headers.append(
                    "Content-Type",
                    HeaderValue::from_str("application/atom+xml").unwrap(),
                );
                (headers, feed.into_replay(schedule).to_string())
                    .into_response()
                    .map(boxed)
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum ReplayError {
    #[error("failed to fetch feed")]
    FetchError(#[from] FetchError),
    #[error("database request failed")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for ReplayError {
    type Body = Body;
    type BodyError = <Self::Body as HttpBody>::Error;

    fn into_response(self) -> Response<Self::Body> {
        tracing::error!(?self);

        let body = match self {
            ReplayError::FetchError(err) => Body::from(err.to_string()),
            ReplayError::DatabaseError(err) => Body::from(err.to_string()),
        };

        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body)
            .expect("Failed to build error response")
    }
}
