use crate::CachedEntry;
use chrono::{DateTime, NaiveDateTime, Utc};

pub fn parse_dt(dt_str: &str) -> DateTime<Utc> {
    let fmt = "%Y-%m-%dT%H:%M:%S";
    let ndt = NaiveDateTime::parse_from_str(dt_str, fmt).unwrap_or_else(|_| {
        panic!(
            "DateTime::parse_from_str(\"{}\", \"{}\") failed with",
            dt_str, fmt
        )
    });
    DateTime::<Utc>::from_utc(ndt, Utc)
}

pub fn cached_entries(feed_id: i64, items: Vec<(&str, &str, &str)>) -> Vec<CachedEntry> {
    items
        .into_iter()
        .map(|(id, published, noticed)| CachedEntry {
            feed_id,
            id: id.to_string(),
            published: if published == "gone" {
                None
            } else {
                Some(parse_dt(published))
            },
            noticed: parse_dt(if noticed == "pub" { published } else { noticed }),
        })
        .collect()
}

#[test]
#[should_panic]
fn invalid_dt() {
    parse_dt("wat");
}
