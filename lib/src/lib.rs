mod diff;
// pub mod feed;
pub mod parser;
mod reader;
mod replay;

#[cfg(test)]
pub mod test_helpers;

use chrono::{DateTime, Utc};
pub use diff::{create_cached_entry_map, diff_feed};
pub use reader::{FeedSummary, FeedSummaryError};
// pub use feed::{Feed, FeedSummary, FeedSummaryItem, ParseFeedError};
pub use replay::{replay_feed, ReplayedItem, Reschedule};

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
