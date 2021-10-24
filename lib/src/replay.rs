use std::collections::HashMap;

use chrono::{DateTime, Utc};
use chronoutil::DateRule;

use crate::FeedSummaryItem;

#[derive(Debug, PartialEq)]
pub struct ReplayedItem<'a> {
    id: &'a str,
    timestamp: DateTime<Utc>,
}

#[derive(Debug)]
struct Scheduled<'a> {
    replayed: bool,
    items: Vec<&'a FeedSummaryItem>,
}

#[derive(Debug)]
enum Unpublished {
    BeforeSlot,
    AfterSlot,
    Never,
}

struct DelayedItems<'a>(Vec<&'a FeedSummaryItem>);
impl<'a> DelayedItems<'a> {
    fn new() -> Self {
        DelayedItems(Vec::new())
    }

    fn add<'b>(&'b mut self, item: &'a FeedSummaryItem) {
        self.0.push(item);
        self.0.sort_by_key(|i| i.published);
    }

    fn pop_eligible<'b>(&'b mut self, slot: DateTime<Utc>) -> Option<&'a FeedSummaryItem> {
        let delayed_index = self.0.iter().position(|i| i.noticed <= slot);
        delayed_index.map(|index| self.0.remove(index))
    }

    fn is_empty(&'a self) -> bool {
        self.0.is_empty()
    }
}

impl<'a> Scheduled<'a> {
    fn rescheduled<'b>(&'a self, slot: DateTime<Utc>, item: &'b FeedSummaryItem) -> bool {
        self.items.len() > 1
            && (self.items.iter()).any(|i| {
                i.noticed <= slot && i.noticed >= item.noticed && i.published > item.published
            })
    }

    fn finally_unpublished(&'a self, slot: DateTime<Utc>) -> Unpublished {
        let item = self.items.iter().max_by_key(|i| i.noticed);
        match item {
            Some(item) if item.published.is_none() => {
                if item.noticed > slot {
                    Unpublished::AfterSlot
                } else {
                    Unpublished::BeforeSlot
                }
            }
            _ => Unpublished::Never,
        }
    }
}

fn init_reschedule_map(items: &[FeedSummaryItem]) -> HashMap<&String, Scheduled> {
    let mut rescheduled = HashMap::new();
    for item in items.iter() {
        let scheduled = rescheduled.entry(&item.id).or_insert(Scheduled {
            replayed: false,
            items: Vec::new(),
        });
        scheduled.items.push(item);
    }
    rescheduled
}

pub fn replay_feed(
    items: &[FeedSummaryItem],
    rule: DateRule<DateTime<Utc>>,
    until: DateTime<Utc>,
) -> Vec<ReplayedItem> {
    let mut rescheduled = init_reschedule_map(&items);
    let mut replayed = Vec::new();
    let some_until = Some(until);
    let mut items = items.iter().filter(|item| item.published < some_until);
    let mut delayed = DelayedItems::new();

    for slot in rule.with_end(until) {
        let some_slot = Some(slot);
        loop {
            let next_up = delayed.pop_eligible(slot).or_else(|| items.next());
            if let Some(next) = next_up {
                match rescheduled.get_mut(&next.id) {
                    Some(scheduled) if !scheduled.replayed => {
                        if next.published < some_slot {
                            if !scheduled.rescheduled(slot, next) {
                                if next.noticed > slot {
                                    delayed.add(&next);
                                    continue;
                                }
                                match scheduled.finally_unpublished(slot) {
                                    Unpublished::BeforeSlot => {
                                        // we found out about the unpublish
                                        // before we got here, so loop
                                        // around and try the next one
                                        continue;
                                    }
                                    Unpublished::AfterSlot => {
                                        // skip this slot since we
                                        // previously replayed this now
                                        // known to be unpublished item
                                        break;
                                    }
                                    Unpublished::Never => {
                                        replayed.push(ReplayedItem {
                                            id: &next.id,
                                            timestamp: slot,
                                        });
                                        scheduled.replayed = true;
                                        break; // move to next slot
                                    }
                                }
                            }
                        } else if let Some(published) = next.published {
                            replayed.push(ReplayedItem {
                                id: &next.id,
                                timestamp: published,
                            });
                            scheduled.replayed = true;
                        }
                    }
                    _ => {}
                }
            } else if delayed.is_empty() {
                return replayed; // ran out of items, don't loop over the rest of the slots
            } else {
                break;
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
                published: if published == "gone" {
                    None
                } else {
                    Some(parse_dt(published))
                },
                noticed: parse_dt(if noticed == "pub" { published } else { noticed }),
            })
            .collect()
    }

    fn replayed_items<'a>(items: Vec<(&'a str, &'a str)>) -> Vec<ReplayedItem<'a>> {
        items
            .into_iter()
            .map(|(id, dt_str)| ReplayedItem {
                id: id,
                timestamp: parse_dt(dt_str),
            })
            .collect()
    }

    #[test]
    fn empty_feed() {
        let items = summary_items(vec![]);
        let result = replay_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-28T21:00:00")),
            parse_dt("2014-12-28T21:00:00"),
        );
        assert_eq!(result, vec![]);
    }

    #[test]
    fn one_item() {
        let items = summary_items(vec![("1", "2013-10-10T21:00:00", "pub")]);
        let result = replay_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-28T21:00:00")),
            parse_dt("2014-12-28T21:00:00"),
        );
        assert_eq!(result, replayed_items(vec![("1", "2014-11-28T21:00:00")]));
    }

    #[test]
    fn two_items() {
        let items = summary_items(vec![
            ("1", "2013-10-10T21:00:00", "pub"),
            ("2", "2013-11-10T21:00:00", "pub"),
        ]);
        let result = replay_feed(
            &items,
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
        let items = summary_items(vec![
            ("1", "2013-11-28T21:00:00", "pub"),
            ("2", "2013-12-28T21:00:00", "pub"),
        ]);
        let result = replay_feed(
            &items,
            DateRule::weekly(parse_dt("2014-11-28T21:00:00")),
            parse_dt("2014-11-28T22:00:00"),
        );
        assert_eq!(result, replayed_items(vec![("1", "2014-11-28T21:00:00")]));
    }

    #[test]
    fn resumes_original_schedule_once_caught_up() {
        let items = summary_items(vec![
            ("1", "2014-11-01T09:00:00", "pub"),
            ("2", "2014-11-04T21:00:00", "pub"),
            ("3", "2014-11-09T22:00:00", "pub"),
            ("4", "2014-11-13T22:00:00", "pub"),
        ]);
        let result = replay_feed(
            &items,
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
        let items = summary_items(vec![
            ("1", "2014-11-01T09:00:00", "pub"),
            ("1", "2014-11-04T21:00:00", "pub"),
        ]);
        let result = replay_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-03T20:00:00")),
            parse_dt("2014-11-12T22:00:00"),
        );
        assert_eq!(result, replayed_items(vec![("1", "2014-11-03T20:00:00"),]));
    }

    #[test]
    fn does_not_schedule_a_replay_if_a_reschedule_is_noticed_before_slot() {
        let items = summary_items(vec![
            ("1", "2014-11-01T09:00:00", "pub"),
            ("2", "2014-11-02T21:00:00", "pub"),
            ("1", "2014-11-04T21:00:00", "pub"),
        ]);
        let result = replay_feed(
            &items,
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

    #[test]
    fn moved_forward_noticed_after_slot() {
        let items = summary_items(vec![
            ("1", "2014-11-01T09:00:00", "pub"),
            ("2", "2014-11-02T21:00:00", "pub"),
            ("1", "2014-11-04T21:00:00", "pub"),
        ]);
        let result = replay_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-03T20:00:00")),
            parse_dt("2014-12-12T22:00:00"),
        );
        assert_eq!(
            result,
            replayed_items(vec![
                ("1", "2014-11-03T20:00:00"),
                ("2", "2014-11-04T20:00:00"),
            ])
        );
    }

    #[test]
    fn moved_forward_noticed_before_slot() {
        let items = summary_items(vec![
            ("1", "2014-11-01T09:00:00", "pub"),
            ("2", "2014-11-02T21:00:00", "pub"),
            ("1", "2014-11-04T21:00:00", "pub"),
        ]);
        let result = replay_feed(
            &items,
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

    #[test]
    fn moved_backward_noticed_before_slot() {
        let items = summary_items(vec![
            ("1", "2014-11-01T21:00:00", "2014-11-08T21:00:00"),
            ("2", "2014-11-02T09:00:00", "pub"),
            ("1", "2014-11-06T21:00:00", "pub"),
        ]);
        let result = replay_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-09T20:00:00")),
            parse_dt("2014-12-12T22:00:00"),
        );
        assert_eq!(
            result,
            replayed_items(vec![
                ("1", "2014-11-09T20:00:00"),
                ("2", "2014-11-10T20:00:00"),
            ])
        );
    }

    #[test]
    fn moved_backward_noticed_after_slot() {
        let items = summary_items(vec![
            ("1", "2014-11-01T21:00:00", "2014-11-08T21:00:00"),
            ("2", "2014-11-02T09:00:00", "pub"),
            ("1", "2014-11-06T20:00:00", "pub"),
        ]);
        let result = replay_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-06T10:00:00")),
            parse_dt("2014-12-12T22:00:00"),
        );
        assert_eq!(
            result,
            replayed_items(vec![
                ("2", "2014-11-06T10:00:00"),
                ("1", "2014-11-07T10:00:00"),
            ])
        );
    }

    #[test]
    fn published_retroactively_noticed_before_slot() {
        let items = summary_items(vec![
            ("1", "2014-11-01T21:00:00", "2014-11-05T21:00:00"),
            ("2", "2014-11-02T09:00:00", "pub"),
        ]);
        let result = replay_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-06T10:00:00")),
            parse_dt("2014-12-12T22:00:00"),
        );
        assert_eq!(
            result,
            replayed_items(vec![
                ("1", "2014-11-06T10:00:00"),
                ("2", "2014-11-07T10:00:00"),
            ])
        );
    }

    #[test]
    fn published_retroactively_noticed_after_slot() {
        let items = summary_items(vec![
            ("1", "2014-11-01T21:00:00", "2014-11-11T10:00:00"),
            ("2", "2014-11-02T09:00:00", "pub"),
        ]);
        let result = replay_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-10T20:00:00")),
            parse_dt("2014-12-12T22:00:00"),
        );
        assert_eq!(
            result,
            replayed_items(vec![
                ("2", "2014-11-10T20:00:00"),
                ("1", "2014-11-11T20:00:00"),
            ])
        );
    }

    #[test]
    fn published_retroactively_noticed_after_slot_and_missed_a_slot() {
        let items = summary_items(vec![
            ("1", "2014-11-01T21:00:00", "2014-11-11T21:00:00"),
            ("2", "2014-11-02T09:00:00", "pub"),
        ]);
        let result = replay_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-10T10:00:00")),
            parse_dt("2014-12-12T22:00:00"),
        );
        assert_eq!(
            result,
            replayed_items(vec![
                ("2", "2014-11-10T10:00:00"),
                // Because the publish wasn't noticed before the slot on the
                // 11th, it doesn't get replayed until the next available slot
                // on the 12th
                ("1", "2014-11-12T10:00:00"),
            ])
        );
    }

    #[test]
    fn unpublish_noticed_after_slot() {
        let items = summary_items(vec![
            ("1", "2014-11-01T21:00:00", "pub"),
            ("2", "2014-11-03T21:00:00", "pub"),
            ("1", "gone", "2014-11-11T21:00:00"),
        ]);
        let result = replay_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-10T10:00:00")),
            parse_dt("2014-12-12T22:00:00"),
        );
        assert_eq!(
            result,
            replayed_items(vec![
                // We didn't notice the unpublish before the slot was filled, so it
                // was replayed briefly, but we need to keep the slot open to avoid
                // shifting anything else around.
                ("2", "2014-11-11T10:00:00"),
            ])
        );
    }

    #[test]
    fn unpublish_noticed_before_slot() {
        let items = summary_items(vec![
            ("1", "2014-11-01T21:00:00", "pub"),
            ("2", "2014-11-03T21:00:00", "pub"),
            ("1", "gone", "2014-11-10T21:00:00"),
        ]);
        let result = replay_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-12T10:00:00")),
            parse_dt("2014-12-12T22:00:00"),
        );
        assert_eq!(
            result,
            replayed_items(vec![
                // We noticed the unpublish before we got to the slot, so we
                // publish the next available episode in that slot
                ("2", "2014-11-12T10:00:00"),
            ])
        );
    }
}
