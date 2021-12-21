use crate::{summarize::SummaryItem, CachedEntry};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

pub fn diff_feed(
    item_map: &HashMap<&str, &SummaryItem>,
    cached_map: &HashMap<&str, &CachedEntry>,
    feed_id: i64,
    now: DateTime<Utc>,
) -> Vec<CachedEntry> {
    let mut updates = Vec::new();

    for (item_id, cached_item) in cached_map {
        let published = item_map.get(item_id).map(|i| i.timestamp);
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
                published: Some(item.timestamp),
                noticed: now,
            });
        }
    }

    updates
}

pub fn create_cached_entry_map(entries: &[CachedEntry]) -> HashMap<&str, &CachedEntry> {
    let mut map: HashMap<&str, &CachedEntry> = HashMap::new();
    for item in entries.iter() {
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

#[cfg(test)]
mod test {
    use super::create_cached_entry_map;
    use crate::summarize::SummaryItem;
    use crate::test_helpers::{cached_entries, parse_dt};
    use crate::{diff_feed, FeedSummary};
    use std::collections::HashMap;

    fn feed(items: Vec<(&str, &str)>) -> FeedSummary {
        let entries = items
            .into_iter()
            .map(|(id, timestamp)| SummaryItem {
                id: id.to_string(),
                title: id.to_string(),
                timestamp: parse_dt(timestamp),
            })
            .collect();
        FeedSummary::from_items(entries)
    }

    #[test]
    fn new_feed() {
        let item_map = HashMap::new();
        let cached_map = HashMap::new();
        let now = parse_dt("2013-10-10T21:00:00");
        let updates = diff_feed(&item_map, &cached_map, 1, now);
        assert_eq!(updates, vec![]);
    }

    #[test]
    fn new_entry() {
        let feed = feed(vec![("1", "2013-10-01T21:00:00")]);
        let item_map = feed.id_map();
        let cached_map = HashMap::new();
        let now = parse_dt("2013-10-10T21:00:00");
        let updates = diff_feed(&item_map, &cached_map, 1, now);
        assert_eq!(
            updates,
            cached_entries(1, vec![("1", "2013-10-01T21:00:00", "2013-10-10T21:00:00")])
        );
    }

    #[test]
    fn updated_entry() {
        let feed = feed(vec![("1", "2013-10-02T21:00:00")]);
        let item_map = feed.id_map();

        let cached = cached_entries(1, vec![("1", "2013-10-01T21:00:00", "2013-10-01T22:00:00")]);
        let cached_map = create_cached_entry_map(&cached);
        let now = parse_dt("2013-10-10T21:00:00");
        let updates = diff_feed(&item_map, &cached_map, 1, now);
        assert_eq!(
            updates,
            cached_entries(1, vec![("1", "2013-10-02T21:00:00", "2013-10-10T21:00:00")])
        );
    }

    #[test]
    fn removed_entry() {
        let feed = feed(vec![]);
        let item_map = feed.id_map();
        let cached = cached_entries(1, vec![("1", "2013-09-01T21:00:00", "2013-09-01T22:00:00")]);
        let cached_map = create_cached_entry_map(&cached);
        let now = parse_dt("2013-10-10T21:00:00");
        let updates = diff_feed(&item_map, &cached_map, 1, now);
        assert_eq!(
            updates,
            cached_entries(1, vec![("1", "gone", "2013-10-10T21:00:00")])
        );
    }

    #[test]
    fn no_change() {
        let feed = feed(vec![("1", "2013-10-02T21:00:00")]);
        let item_map = feed.id_map();
        let cached = cached_entries(1, vec![("1", "2013-10-02T21:00:00", "2013-09-01T22:00:00")]);
        let cached_map = create_cached_entry_map(&cached);
        let now = parse_dt("2013-10-10T21:00:00");
        let updates = diff_feed(&item_map, &cached_map, 1, now);
        assert_eq!(updates, cached_entries(1, vec![]));
    }
}
