use chrono::{DateTime, Utc};
use feed_rs::{model::Feed as ParsedFeed, parser};
use serde::Serialize;

pub struct Feed {
    feed: ParsedFeed,
    uri: Option<String>,
}

impl Feed {}

impl Feed {
    fn new(feed: ParsedFeed, uri: Option<String>) -> Self {
        Feed { feed, uri }
    }

    pub fn from_source(source: &[u8], uri: Option<&str>) -> Self {
        let feed = parser::parse_with_uri(source, uri).unwrap();
        Feed::new(feed, uri.map(|uri| uri.to_string()))
    }
}

#[derive(Serialize)]
pub struct FeedSummary {
    pub title: Option<String>,
    pub uri: Option<String>,
    pub items: Vec<FeedSummaryItem>,
}

#[derive(Serialize, Clone, Debug)]
pub struct FeedSummaryItem {
    pub id: String,
    pub title: Option<String>,
    pub published: DateTime<Utc>,
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
                    published: timestamp,
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
