use chrono::{DateTime, NaiveDateTime, Utc};
use podreplay_lib::FeedSummaryItem;

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

pub fn summary_items(items: Vec<(&str, &str, &str)>) -> Vec<FeedSummaryItem> {
    items
        .into_iter()
        .map(|(id, published, noticed)| FeedSummaryItem {
            id: id.to_string(),
            title: None,
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
