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
    for slot in rule.with_end(until) {
        match items.next() {
            Some(next) => {
                if next.published < slot {
                    replayed.push(ReplayedItem {
                        id: next.id,
                        timestamp: slot,
                    })
                } else {
                    replayed.push(ReplayedItem {
                        id: next.id,
                        timestamp: next.published,
                    });
                    while let Some(next) = items.next() {
                        replayed.push(ReplayedItem {
                            id: next.id,
                            timestamp: next.published,
                        });
                    }
                }
            }
            None => return replayed,
        }
    }
    replayed
}

#[cfg(test)]
mod tests {
    use chrono::{DateTime, Utc};
    use chronoutil::DateRule;

    use crate::{
        replay::{replay_feed, ReplayedItem},
        FeedSummaryItem,
    };

    fn parse_dt(dt_str: &str) -> DateTime<Utc> {
        DateTime::parse_from_rfc3339(dt_str)
            .unwrap()
            .with_timezone(&Utc)
    }

    fn summary_items(items: Vec<(&str, &str)>) -> Vec<FeedSummaryItem> {
        items
            .into_iter()
            .map(|(id, dt_str)| {
                let dt = parse_dt(dt_str);
                FeedSummaryItem {
                    id: id.to_string(),
                    title: Some(id.to_string()),
                    published: dt,
                    noticed: dt,
                }
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

    fn daily(start: DateTime<Utc>, end: DateTime<Utc>) -> DateRule<DateTime<Utc>> {
        DateRule::daily(start).with_end(end)
    }

    #[test]
    fn empty_feed() {
        let feed = summary_items(vec![]);
        let result = replay_feed(
            feed,
            DateRule::daily(parse_dt("2014-11-28T21:00:00+00:00")),
            parse_dt("2014-12-28T21:00:00+00:00"),
        );
        assert_eq!(result, vec![]);
    }

    #[test]
    fn one_item() {
        let feed = summary_items(vec![("1", "2013-10-10T21:00:00+00:00")]);
        let result = replay_feed(
            feed,
            DateRule::daily(parse_dt("2014-11-28T21:00:00+00:00")),
            parse_dt("2014-12-28T21:00:00+00:00"),
        );
        assert_eq!(
            result,
            replayed_items(vec![("1", "2014-11-28T21:00:00+00:00")])
        );
    }

    #[test]
    fn two_items() {
        let feed = summary_items(vec![
            ("1", "2013-10-10T21:00:00+00:00"),
            ("2", "2013-11-10T21:00:00+00:00"),
        ]);
        let result = replay_feed(
            feed,
            DateRule::daily(parse_dt("2014-11-28T21:00:00+00:00")),
            parse_dt("2014-12-28T21:00:00+00:00"),
        );
        assert_eq!(
            result,
            replayed_items(vec![
                ("1", "2014-11-28T21:00:00+00:00"),
                ("2", "2014-11-29T21:00:00+00:00")
            ])
        );
    }

    #[test]
    fn stops_repeating_at_end() {
        let feed = summary_items(vec![
            ("1", "2013-11-28T21:00:00+00:00"),
            ("2", "2013-12-28T21:00:00+00:00"),
        ]);
        let result = replay_feed(
            feed,
            DateRule::weekly(parse_dt("2014-11-28T21:00:00+00:00")),
            parse_dt("2014-11-28T22:00:00+00:00"),
        );
        assert_eq!(
            result,
            replayed_items(vec![("1", "2014-11-28T21:00:00+00:00")])
        );
    }

    #[test]
    fn resumes_original_schedule_once_caught_up() {
        let feed = summary_items(vec![
            ("1", "2014-11-01T09:00:00+00:00"),
            ("2", "2014-11-04T21:00:00+00:00"),
            ("3", "2014-11-09T22:00:00+00:00"),
            ("4", "2014-11-13T22:00:00+00:00"),
        ]);
        let result = replay_feed(
            feed,
            DateRule::daily(parse_dt("2014-11-03T20:00:00+00:00")),
            parse_dt("2014-11-12T22:00:00+00:00"),
        );
        assert_eq!(
            result,
            replayed_items(vec![
                ("1", "2014-11-03T20:00:00+00:00"),
                ("2", "2014-11-04T21:00:00+00:00"),
                ("3", "2014-11-09T22:00:00+00:00"),
            ])
        );
    }
}
