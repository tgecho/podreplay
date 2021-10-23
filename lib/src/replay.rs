use chrono::{DateTime, Utc};
use chronoutil::DateRule;

use crate::FeedSummaryItem;

#[derive(Debug, PartialEq)]
pub struct ReplayedItem {
    id: String,
    timestamp: DateTime<Utc>,
}

// TODO: Consider an option to stay on replay schedule after catching up, even if it means skipping days
// TODO: Add handling for rescheduled episodes
pub fn replay_feed(
    items: Vec<FeedSummaryItem>,
    rule: DateRule<DateTime<Utc>>,
    until: DateTime<Utc>,
) -> Vec<ReplayedItem> {
    let mut replayed = Vec::new();
    let mut items = items.into_iter().take_while(|item| item.published < until);
    let mut replay = |id, timestamp| replayed.push(ReplayedItem { id, timestamp });
    for slot in rule.with_end(until) {
        match items.next() {
            Some(next) => {
                if next.published < slot {
                    replay(next.id, slot);
                } else {
                    replay(next.id, next.published);
                    while let Some(next) = items.next() {
                        replay(next.id, next.published);
                    }
                    break;
                }
            }
            None => break,
        }
    }
    replayed
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, NaiveDateTime, Utc};
    use chronoutil::DateRule;

    use crate::{
        replay::{replay_feed, ReplayedItem},
        FeedSummaryItem,
    };

    fn parse_dt(dt_str: &str) -> DateTime<Utc> {
        let fmt = "%Y-%m-%dT%H:%M:%S";
        let ndt = NaiveDateTime::parse_from_str(dt_str, fmt).expect(&format!(
            "DateTime::parse_from_str(\"{}\", \"{}\") failed with",
            dt_str, fmt
        ));
        DateTime::<Utc>::from_utc(ndt, Utc)
    }

    fn summary_items(items: Vec<(&str, &str, &str)>) -> Vec<FeedSummaryItem> {
        items
            .into_iter()
            .map(|(id, published, noticed)| FeedSummaryItem {
                id: id.to_string(),
                title: Some(id.to_string()),
                published: parse_dt(published),
                noticed: parse_dt(if noticed == "pub" { published } else { noticed }),
            })
            .collect()
    }

    fn replayed_items(items: Vec<(&str, &str)>) -> Vec<ReplayedItem> {
        items
            .into_iter()
            .map(|(id, dt_str)| ReplayedItem {
                id: id.to_string(),
                timestamp: parse_dt(dt_str),
            })
            .collect()
    }

    #[test]
    fn empty_feed() {
        let feed = summary_items(vec![]);
        let result = replay_feed(
            feed,
            DateRule::daily(parse_dt("2014-11-28T21:00:00")),
            parse_dt("2014-12-28T21:00:00"),
        );
        assert_eq!(result, vec![]);
    }

    #[test]
    fn one_item() {
        let feed = summary_items(vec![("1", "2013-10-10T21:00:00", "pub")]);
        let result = replay_feed(
            feed,
            DateRule::daily(parse_dt("2014-11-28T21:00:00")),
            parse_dt("2014-12-28T21:00:00"),
        );
        assert_eq!(result, replayed_items(vec![("1", "2014-11-28T21:00:00")]));
    }

    #[test]
    fn two_items() {
        let feed = summary_items(vec![
            ("1", "2013-10-10T21:00:00", "pub"),
            ("2", "2013-11-10T21:00:00", "pub"),
        ]);
        let result = replay_feed(
            feed,
            DateRule::daily(parse_dt("2014-11-28T21:00:00")),
            parse_dt("2014-12-28T21:00:00"),
        );
        assert_eq!(
            result,
            replayed_items(vec![
                ("1", "2014-11-28T21:00:00"),
                ("2", "2014-11-29T21:00:00")
            ])
        );
    }

    #[test]
    fn stops_repeating_at_end() {
        let feed = summary_items(vec![
            ("1", "2013-11-28T21:00:00", "pub"),
            ("2", "2013-12-28T21:00:00", "pub"),
        ]);
        let result = replay_feed(
            feed,
            DateRule::weekly(parse_dt("2014-11-28T21:00:00")),
            parse_dt("2014-11-28T22:00:00"),
        );
        assert_eq!(result, replayed_items(vec![("1", "2014-11-28T21:00:00")]));
    }

    #[test]
    fn resumes_original_schedule_once_caught_up() {
        let feed = summary_items(vec![
            ("1", "2014-11-01T09:00:00", "pub"),
            ("2", "2014-11-04T21:00:00", "pub"),
            ("3", "2014-11-09T22:00:00", "pub"),
            ("4", "2014-11-13T22:00:00", "pub"),
        ]);
        let result = replay_feed(
            feed,
            DateRule::daily(parse_dt("2014-11-03T20:00:00")),
            parse_dt("2014-11-12T22:00:00"),
        );
        assert_eq!(
            result,
            replayed_items(vec![
                ("1", "2014-11-03T20:00:00"),
                ("2", "2014-11-04T21:00:00"),
                ("3", "2014-11-09T22:00:00"),
            ])
        );
    }
}
