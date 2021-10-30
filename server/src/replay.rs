#![allow(clippy::large_enum_variant)]

use axum::{
    body::Body,
    extract::{Extension, Query},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use chronoutil::DateRule;
use headers::{HeaderMap, HeaderValue};
use hyper::{Response, StatusCode};
use lazy_static::lazy_static;
use podreplay_lib::{
    diff_feed, feed::create_cached_entry_map, replay_feed, Feed, ParseFeedError, ReplayedItem,
};
use regex::Regex;
use serde::Deserialize;
use thiserror::Error;

use crate::db::Db;

#[derive(Deserialize, Debug)]
pub struct SummaryQuery {
    start: DateTime<Utc>,
    uri: String,
    now: Option<DateTime<Utc>>,
}

lazy_static! {
    static ref ETAG_RE: Regex = Regex::new(r#"^(?:W/)?"?([^"]+)"?$"#).unwrap();
}

fn parse_rfc3339(dt: &str) -> Result<DateTime<Utc>, chrono::ParseError> {
    Ok(DateTime::parse_from_rfc3339(dt)?.into())
}

fn parse_etag(inm: &str) -> Option<&str> {
    let cap = ETAG_RE.captures(inm)?;
    cap.get(1).map(|g| g.as_str())
}

pub async fn get<'a>(
    query: Query<SummaryQuery>,
    headers: HeaderMap,
    Extension(db): Extension<Db>,
) -> Result<ReplayResponse, ReplayError> {
    // TODO: Do we always want to leave this overriding in place? Should we
    // consider not using it to (for example) update the DB to avoid breaking
    // the integrity of something?
    let now = query.now.unwrap_or_else(Utc::now);

    let if_none_match = headers
        .get("if-none-match")
        .and_then(|inm| inm.to_str().ok());

    let split_inm = if_none_match
        .and_then(parse_etag)
        .map(|cap| match cap.split_once('|') {
            Some((expires, etag)) => (parse_rfc3339(expires).ok(), etag),
            None => (None, cap),
        });
    let inm_feed_etag = split_inm.map(|inm| inm.1);
    let inm_expires = split_inm.and_then(|inm| inm.0);

    if let Some(expires) = inm_expires {
        if now < expires {
            return Ok(ReplayResponse::NotModified);
        }
    }

    let fetched = fetch_feed(
        &query.uri,
        inm_feed_etag.map(|etag| format!(r#""{}""#, etag)),
    )
    .await?;

    let (feed, fetched_etag) = match fetched {
        FeedResponse::NotModified => return Ok(ReplayResponse::NotModified),
        FeedResponse::Fetched(feed, fetched_etag) => (feed, fetched_etag),
    };

    let feed_meta = db.update_feed_meta(&query.uri, &now, &fetched_etag).await?;

    let cached_entries = db.get_entries(feed_meta.id).await?;
    let cached_entry_map = create_cached_entry_map(&cached_entries);

    let changes = diff_feed(&feed.id_map(), &cached_entry_map, feed_meta.id, now);

    let entries = if changes.is_empty() {
        cached_entries
    } else {
        db.update_cached_entries(feed_meta.id, &changes).await?
    };

    let rule = DateRule::weekly(query.start);

    let (replayed, next_slot) =
        replay_feed(&entries, rule, query.start, now, feed_meta.first_fetched);

    // TODO: forward on any other safe/relevant feed caching related headers?
    let mut headers = HeaderMap::new();
    if let Some(expires) = next_slot.map(|dt| dt.to_rfc2822()) {
        headers.insert("Expires", HeaderValue::from_str(&expires).unwrap());
    }
    if let Some(feed_etag) = fetched_etag.and_then(|e| Some(parse_etag(&e)?.to_string())) {
        let etag = format!(r#""{}|{}""#, next_slot.unwrap().to_rfc3339(), feed_etag);
        headers.insert("Etag", HeaderValue::from_str(&etag).unwrap());
    }

    // TODO: use replayed and the fetched feed to build a final feed
    Ok(ReplayResponse::Replay {
        feed,
        schedule: replayed,
        headers,
    })
}

enum FeedResponse {
    NotModified,
    Fetched(Feed, Option<String>),
}

async fn fetch_feed(uri: &str, etag: Option<String>) -> Result<FeedResponse, ReplayError> {
    let client = reqwest::Client::builder().build()?;
    let req = client.get(uri).header("User-Agent", "podreplay/0.1");
    let req = if let Some(etag) = etag {
        req.header("If-None-Match", etag)
    } else {
        req
    };
    let resp = req.send().await?;

    if resp.status() == StatusCode::NOT_MODIFIED {
        return Ok(FeedResponse::NotModified);
    }

    let etag = resp
        .headers()
        .get("etag")
        .and_then(|etag| etag.to_str().ok())
        .map(|etag| etag.to_string());

    let body = resp.bytes().await?;
    let feed = Feed::from_source(&body, Some(uri))?;

    Ok(FeedResponse::Fetched(feed, etag))
}

pub enum ReplayResponse {
    NotModified,
    Replay {
        headers: HeaderMap,
        feed: Feed,
        schedule: Vec<ReplayedItem>,
    },
}

impl IntoResponse for ReplayResponse {
    type Body = axum::body::Full<axum::body::Bytes>;
    type BodyError = <Self::Body as axum::body::HttpBody>::Error;

    fn into_response(self) -> Response<Self::Body> {
        match self {
            ReplayResponse::NotModified => (StatusCode::NOT_MODIFIED, Json("")).into_response(),
            ReplayResponse::Replay {
                headers,
                feed,
                schedule,
            } => (headers, Json(schedule)).into_response(),
        }
    }
}

#[derive(Error, Debug)]
pub enum ReplayError {
    #[error("failed to fetch feed")]
    FetchFeedFailed(#[from] reqwest::Error),
    #[error("failed to parse feed")]
    ParseFeedFailed(#[from] ParseFeedError),
    #[error("database request failed")]
    DatabaseError(#[from] sqlx::Error),
}

impl IntoResponse for ReplayError {
    type Body = Body;
    type BodyError = <Self::Body as axum::body::HttpBody>::Error;

    fn into_response(self) -> Response<Self::Body> {
        let body = match self {
            ReplayError::FetchFeedFailed(err) => Body::from(err.to_string()),
            ReplayError::ParseFeedFailed(err) => Body::from(err.to_string()),
            ReplayError::DatabaseError(err) => Body::from(err.to_string()),
        };

        Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(body)
            .unwrap()
    }
}