use axum::{
    body::Body,
    extract::{Extension, Query},
    response::IntoResponse,
    Json,
};
use chrono::{DateTime, Utc};
use chronoutil::DateRule;
use hyper::{Response, StatusCode};
use podreplay_lib::{diff_feed, feed::create_cached_entry_map, replay_feed, Feed, ParseFeedError};
use serde::Deserialize;
use thiserror::Error;

use crate::db::Db;

#[derive(Deserialize, Debug)]
pub struct SummaryQuery {
    start: DateTime<Utc>,
    uri: String,
    now: Option<DateTime<Utc>>,
}

pub async fn get<'a>(
    query: Query<SummaryQuery>,
    Extension(db): Extension<Db>,
) -> Result<Json<Vec<podreplay_lib::ReplayedItem>>, ReplayError> {
    // TODO: Do we always want to leave this overriding in place? Should we
    // consider not using it to (for example) update the DB to avoid breaking
    // the integrity of something?
    let now = query.now.unwrap_or_else(Utc::now);

    // TODO: use this?
    // let feed_meta = db.get_feed_meta(&query.uri).await?;

    // TODO: break etag (if it exists) into [feed etag] and [next scheduled]
    // parts. I need to think about it a bit more, but we may be able to
    // immediately return a 304 if we haven't hit the next scheduled timestamp.
    // If that's not safe, we should be able to at least check the upstream
    // podcast's etag and potentially still send a 304. Note that if we do use
    // the feed's etag, we'll still need to be aware of whether we're likely to
    // need to add another episode. It doesn't do us much good if we get back a
    // 304 but need the feed to generate a new version with the latest scheduled
    // episode.

    let feed = fetch_feed(&query.uri).await?;
    let feed_id = db
        .update_feed_meta(&query.uri, &now, Some("TODO: etag"))
        .await?;

    let cached_entries = db.get_entries(feed_id).await?;
    let cached_entry_map = create_cached_entry_map(&cached_entries);

    let changes = diff_feed(&feed.id_map(), &cached_entry_map, feed_id, now);

    let entries = if changes.is_empty() {
        cached_entries
    } else {
        // TODO: IF this is a new feed, consider defaulting "noticed" to the
        // published date or replaying can get weird. In theory this shouldn't
        // matter since by definition anyone who starts a replay will only be
        // moving forward. If we give the ability to pick a "podcast start",
        // that shouldn't affect anything either.
        db.update_cached_entries(feed_id, &changes).await?
    };

    let rule = DateRule::weekly(query.start);

    let replayed = replay_feed(&entries, rule, query.start, now);

    // TODO: use replayed and the fetched feed to build a final feed
    Ok(Json(replayed))
}

async fn fetch_feed(uri: &str) -> Result<Feed, ReplayError> {
    let client = reqwest::Client::builder().build()?;
    let resp = client
        .get(uri)
        .header("User-Agent", "podreplay/0.1")
        .send()
        .await?;
    let body = resp.bytes().await?;
    let feed = Feed::from_source(&body, Some(uri))?;
    Ok(feed)
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
