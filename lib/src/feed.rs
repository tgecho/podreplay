use std::collections::HashMap;

use chrono::{DateTime, Utc};
pub use feed_rs::parser::ParseFeedError;
use feed_rs::{
    model::{Entry, Feed as ParsedFeed},
    parser::{self},
};
use serde::Serialize;

pub struct Feed {
    feed: ParsedFeed,
    uri: Option<String>,
}

impl Feed {
    pub fn new(feed: ParsedFeed, uri: Option<String>) -> Self {
        Feed { feed, uri }
    }

    pub fn from_source(source: &[u8], uri: Option<&str>) -> Result<Self, ParseFeedError> {
        let parsed = parser::parse_with_uri(source, uri)?;
        let feed = Feed::new(parsed, uri.map(|uri| uri.to_string()));
        Ok(feed)
    }

    pub fn id_map(&self) -> HashMap<&str, &Entry> {
        self.feed
            .entries
            .iter()
            .map(|e| (e.id.as_str(), e))
            .collect()
    }
}

#[derive(Serialize)]
pub struct FeedSummary {
    pub title: Option<String>,
    pub uri: Option<String>,
    pub items: Vec<FeedSummaryItem>,
}

impl FeedSummary {
    pub fn create_cache_map(&self) -> HashMap<&str, &FeedSummaryItem> {
        let mut map: HashMap<&str, &FeedSummaryItem> = HashMap::new();
        for item in self.items.iter() {
            match map.get(item.id.as_str()) {
                Some(entry) => {
                    if item.noticed > entry.noticed {
                        map.insert(&item.id, item);
                    }
                }
                None => {
                    map.insert(&item.id, item);
                }
            }
        }
        map
    }
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct FeedSummaryItem {
    pub id: String,
    pub title: Option<String>,
    pub published: Option<DateTime<Utc>>,
    pub noticed: DateTime<Utc>,
}

impl From<Feed> for FeedSummary {
    fn from(feed: Feed) -> Self {
        let title = feed.feed.title.map(|t| t.content);
        let mut items = Vec::with_capacity(feed.feed.entries.len());
        for entry in feed.feed.entries {
            if let Some(timestamp) = entry.published.or(entry.updated) {
                items.push(FeedSummaryItem {
                    id: entry.id,
                    title: entry.title.map(|t| t.content),
                    published: Some(timestamp),
                    noticed: timestamp,
                });
            }
        }
        items.reverse();
        items.sort_by_key(|i| i.published);
        FeedSummary {
            title,
            uri: feed.uri,
            items,
        }
    }
}
