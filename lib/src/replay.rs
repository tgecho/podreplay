use std::collections::HashMap;

use chrono::{DateTime, Utc};
use chronoutil::DateRule;

use crate::FeedSummaryItem;

#[derive(Debug, PartialEq)]
pub struct ReplayedItem {
    id: String,
    timestamp: DateTime<Utc>,
}

#[derive(Debug)]
struct Scheduled<'a> {
    replayed: bool,
    items: Vec<&'a FeedSummaryItem>,
}

// TODO: Consider an option to stay on replay schedule after catching up, even if it means skipping days
// TODO: Add handling for rescheduled episodes
pub fn replay_feed(
    items: Vec<FeedSummaryItem>,
    rule: DateRule<DateTime<Utc>>,
    until: DateTime<Utc>,
) -> Vec<ReplayedItem> {
    let mut rescheduled = HashMap::new();
    for item in items.iter() {
        let scheduled = rescheduled.entry(&item.id).or_insert(Scheduled {
            replayed: false,
            items: Vec::new(),
        });
        scheduled.items.push(item);
    }

    let mut replayed = Vec::new();
    let mut items = items.iter().take_while(|item| item.published < until);
    let mut replay = |id, timestamp| replayed.push(ReplayedItem { id, timestamp });
    for slot in rule.with_end(until) {
        loop {
            if let Some(next) = items.next() {
                match rescheduled.get_mut(&next.id) {
                    Some(scheduled) if !scheduled.replayed => {
                        if next.published < slot {
                            if scheduled.items.len() > 1
                                && (scheduled.items.iter())
                                    .any(|i| i.noticed < slot && i.published > next.published)
                            {
                                continue;
                            }
                            replay(next.id.clone(), slot);
                            scheduled.replayed = true;
                            break; // move to next slot
                        } else {
                            replay(next.id.clone(), next.published);
                            scheduled.replayed = true;
                        }
                    }
                    _ => {}
                }
            } else {
                return replayed; // ran out of items, don't loop over the rest of the slots
            }
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

    #[test]
    fn does_not_duplicate_a_rescheduled_item_that_already_played() {
        let feed = summary_items(vec![
            ("1", "2014-11-01T09:00:00", "pub"),
            ("1", "2014-11-04T21:00:00", "pub"),
        ]);
        let result = replay_feed(
            feed,
            DateRule::daily(parse_dt("2014-11-03T20:00:00")),
            parse_dt("2014-11-12T22:00:00"),
        );
        assert_eq!(result, replayed_items(vec![("1", "2014-11-03T20:00:00"),]));
    }

    #[test]
    fn does_not_schedule_a_replay_if_a_reschedule_is_noticed_before_slot() {
        let feed = summary_items(vec![
            ("1", "2014-11-01T09:00:00", "pub"),
            ("2", "2014-11-02T21:00:00", "pub"),
            ("1", "2014-11-04T21:00:00", "pub"),
        ]);
        let result = replay_feed(
            feed,
            DateRule::daily(parse_dt("2014-11-06T20:00:00")),
            parse_dt("2014-12-12T22:00:00"),
        );
        assert_eq!(
            result,
            replayed_items(vec![
                ("2", "2014-11-06T20:00:00"),
                ("1", "2014-11-07T20:00:00"),
            ])
        );
    }
}
