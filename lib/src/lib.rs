mod diff;
mod reschedule;
mod rewrite;
mod summarize;

#[cfg(test)]
pub mod test_helpers;

use chrono::{DateTime, Utc};
pub use diff::{create_cached_entry_map, diff_feed};
pub use reschedule::{reschedule_feed, Reschedule};
pub use rewrite::{rewrite_feed, RewriteError};
pub use summarize::{FeedSummary, SummarizeError, SummaryItem};

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
