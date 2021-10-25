use crate::FeedSummaryItem;
use chrono::{DateTime, Utc};
use feed_rs::model::Entry;
use std::collections::HashMap;

pub fn diff_feed(
    item_map: &HashMap<&str, &Entry>,
    cached_map: &HashMap<&str, &FeedSummaryItem>,
    now: DateTime<Utc>,
) -> Vec<FeedSummaryItem> {
    let mut updates = Vec::new();

    for (item_id, cached_item) in cached_map {
        let published = item_map.get(item_id).and_then(|i| i.published);
        if published != cached_item.published {
            updates.push(FeedSummaryItem {
                id: cached_item.id.clone(),
                title: cached_item.title.clone(),
                published,
                noticed: now,
            });
        }
    }

    for (item_id, item) in item_map {
        if !cached_map.contains_key(item_id) {
            updates.push(FeedSummaryItem {
                id: item.id.clone(),
                title: item.title.clone().map(|t| t.content),
                published: item.published,
                noticed: now,
            });
        }
    }

    updates
}
