mod diff;
pub mod feed;
mod replay;

use chrono::{DateTime, Utc};
pub use diff::diff_feed;
pub use feed::{Feed, FeedSummary, FeedSummaryItem, ParseFeedError};
pub use replay::{replay_feed, ReplayedItem};

#[derive(Debug)]
pub struct FeedMeta {
    pub id: i64,
    pub uri: String,
    pub first_fetched: DateTime<Utc>,
    pub last_fetched: DateTime<Utc>,
    pub etag: Option<String>,
}

#[derive(Debug, PartialEq)]
pub struct CachedEntry {
    pub id: String,
    pub feed_id: i64,
    pub noticed: DateTime<Utc>,
    pub published: Option<DateTime<Utc>>,
}
