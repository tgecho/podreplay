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

#[cfg(test)]
mod test {
    use crate::feed::create_cached_entry_map;
    use crate::test_helpers::{cached_entries, parse_dt};
    use crate::{diff_feed, Feed};
    use feed_rs::model::{Entry, Feed as ParsedFeed};
    use std::collections::HashMap;

    fn entry(id: &str, published: &str) -> Entry {
        Entry {
            id: id.to_string(),
            title: None,
            updated: None,
            authors: Vec::new(),
            content: None,
            links: Vec::new(),
            summary: None,
            categories: Vec::new(),
            contributors: Vec::new(),
            published: Some(parse_dt(published)),
            source: None,
            rights: None,
            media: Vec::new(),
        }
    }

    fn feed(items: Vec<(&str, &str)>) -> Feed {
        let entries = items
            .into_iter()
            .map(|(id, published)| entry(id, published))
            .collect();
        Feed::new(
            ParsedFeed {
                feed_type: feed_rs::model::FeedType::Atom,
                id: "id".to_string(),
                title: None,
                updated: None,
                authors: Vec::new(),
                description: None,
                links: Vec::new(),
                categories: Vec::new(),
                contributors: Vec::new(),
                generator: None,
                icon: None,
                language: None,
                logo: None,
                published: None,
                rating: None,
                rights: None,
                ttl: None,
                entries,
            },
            None,
        )
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
