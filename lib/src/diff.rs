use crate::CachedEntry;
use chrono::{DateTime, Utc};
use feed_rs::model::Entry;
use std::collections::HashMap;

pub fn diff_feed(
    item_map: &HashMap<&str, &Entry>,
    cached_map: &HashMap<&str, &CachedEntry>,
    feed_id: i64,
    now: DateTime<Utc>,
) -> Vec<CachedEntry> {
    let mut updates = Vec::new();

    for (item_id, cached_item) in cached_map {
        let published = item_map.get(item_id).and_then(|i| i.published);
        if published != cached_item.published {
            updates.push(CachedEntry {
                id: cached_item.id.clone(),
                feed_id,
                published,
                noticed: now,
            });
        }
    }

    for (item_id, item) in item_map {
        if !cached_map.contains_key(item_id) {
            updates.push(CachedEntry {
                id: item.id.clone(),
                feed_id,
                published: item.published,
                noticed: now,
            });
        }
    }

    updates
}
