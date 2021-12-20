use crate::CachedEntry;
use chrono::{DateTime, Utc};
use chronoutil::DateRule;
use std::collections::HashMap;

pub type Reschedule = HashMap<String, DateTime<Utc>>;

pub fn reschedule_feed(
    items: &[CachedEntry],
    rule: DateRule<DateTime<Utc>>,
    start: DateTime<Utc>,
    now: DateTime<Utc>,
    feed_noticed: DateTime<Utc>,
) -> (Reschedule, Option<DateTime<Utc>>) {
    let mut published_before_cutoff = items
        .iter()
        .filter(|item| item.published.map_or(false, |p| p <= now));
    let mut instances_by_id = create_instances_by_id(items);
    let mut delayed = DelayedItems::new();
    let mut results = HashMap::new();

    for slot in rule {
        if slot >= now {
            return (results, Some(slot));
        }
        let some_slot = Some(slot);
        loop {
            let next_item = delayed
                .pop_eligible(slot)
                .or_else(|| published_before_cutoff.next());
            if let Some(item) = next_item {
                if let Some(instances) = instances_by_id.get_mut(&item.id) {
                    if instances.already_replayed {
                        continue; // try another item
                    }
                    if item.published <= some_slot {
                        if item.noticed > slot && start >= feed_noticed {
                            delayed.add(item);
                            continue; // was published retroactively AFTER we replayed in this slot
                        }
                        if instances.rescheduled_before(slot, item) {
                            continue; // rescheduled into the future, try another item in this slot
                        }
                        match instances.finally_unpublished(slot) {
                            Unpublished::BeforeSlot => {
                                continue; // we found out about this in time to fill the slot with something else
                            }
                            Unpublished::AfterSlot => {
                                break; // we've already replayed this item here, so we need to keep the slot empty
                            }
                            Unpublished::Never => {
                                results.insert(item.id.clone(), slot);
                                instances.already_replayed = true;
                                break; // slot filled, move to the next
                            }
                        }
                    } else if let Some(published) = item.published {
                        // This was published after this slot, meaning we've apparently caught up.
                        // Keep replaying items at their original publication times.
                        results.insert(item.id.clone(), published);
                        instances.already_replayed = true;
                    }
                }
            } else if delayed.is_empty() {
                return (results, None); // ran out of items, don't loop over the rest of the slots
            } else {
                break; // no eligible items available for this slot, try the next
            }
        }
    }
    (results, None)
}

fn create_instances_by_id(items: &[CachedEntry]) -> HashMap<&String, Scheduled> {
    let mut rescheduled = HashMap::new();
    for item in items.iter() {
        let scheduled = rescheduled.entry(&item.id).or_insert(Scheduled {
            already_replayed: false,
            items: Vec::new(),
        });
        scheduled.items.push(item);
    }
    rescheduled
}

struct Scheduled<'a> {
    already_replayed: bool,
    items: Vec<&'a CachedEntry>,
}

enum Unpublished {
    BeforeSlot,
    AfterSlot,
    Never,
}

impl<'a> Scheduled<'a> {
    fn rescheduled_before<'b>(&'a self, slot: DateTime<Utc>, item: &'b CachedEntry) -> bool {
        self.items.len() > 1
            && (self.items.iter()).any(|i| {
                i.noticed <= slot && i.noticed >= item.noticed && i.published > item.published
            })
    }

    fn finally_unpublished(&self, slot: DateTime<Utc>) -> Unpublished {
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

#[derive(Debug)]
struct DelayedItems<'a>(Vec<&'a CachedEntry>);
impl<'a> DelayedItems<'a> {
    fn new() -> Self {
        DelayedItems(Vec::new())
    }

    fn add<'b>(&'b mut self, item: &'a CachedEntry) {
        self.0.push(item);
        self.0.sort_by_key(|i| i.published);
    }

    fn pop_eligible<'b>(&'b mut self, slot: DateTime<Utc>) -> Option<&'a CachedEntry> {
        let delayed_index = self.0.iter().position(|i| i.noticed <= slot);
        delayed_index.map(|index| self.0.remove(index))
    }

    fn is_empty(&'a self) -> bool {
        self.0.is_empty()
    }
}

#[cfg(test)]
mod test {
    use chronoutil::DateRule;
    use std::collections::HashMap;

    use crate::test_helpers::{cached_entries, parse_dt};
    use crate::{reschedule_feed, Reschedule};

    fn replayed_items<'a>(items: Vec<(&'a str, &'a str)>) -> Reschedule {
        items
            .into_iter()
            .map(|(id, dt_str)| (id.to_string(), parse_dt(dt_str)))
            .collect()
    }

    #[test]
    fn empty_feed() {
        let items = cached_entries(1, vec![]);
        let result = reschedule_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-28T21:00:00")),
            parse_dt("2014-11-28T21:00:00"),
            parse_dt("2014-12-28T21:00:00"),
            parse_dt("2014-12-28T21:00:00"),
        );
        assert_eq!(result, (HashMap::new(), None));
    }

    #[test]
    fn one_item() {
        let items = cached_entries(1, vec![("1", "2013-10-10T21:00:00", "pub")]);
        let result = reschedule_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-28T21:00:00")),
            parse_dt("2014-11-28T21:00:00"),
            parse_dt("2014-12-28T21:00:00"),
            parse_dt("2014-12-28T21:00:00"),
        );
        assert_eq!(
            result,
            (replayed_items(vec![("1", "2014-11-28T21:00:00")]), None)
        );
    }

    #[test]
    fn two_items() {
        let items = cached_entries(
            1,
            vec![
                ("1", "2013-10-10T21:00:00", "pub"),
                ("2", "2013-11-10T21:00:00", "pub"),
            ],
        );
        let result = reschedule_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-28T21:00:00")),
            parse_dt("2014-11-28T21:00:00"),
            parse_dt("2014-12-28T21:00:00"),
            parse_dt("2014-12-28T21:00:00"),
        );
        assert_eq!(
            result,
            (
                replayed_items(vec![
                    ("1", "2014-11-28T21:00:00"),
                    ("2", "2014-11-29T21:00:00")
                ]),
                None
            )
        );
    }

    #[test]
    fn stops_repeating_at_end() {
        let items = cached_entries(
            1,
            vec![
                ("1", "2013-11-28T21:00:00", "pub"),
                ("2", "2013-12-04T21:00:00", "pub"),
            ],
        );
        let result = reschedule_feed(
            &items,
            DateRule::weekly(parse_dt("2014-11-28T21:00:00")),
            parse_dt("2014-11-28T21:00:00"),
            parse_dt("2014-11-28T22:00:00"),
            parse_dt("2014-11-28T22:00:00"),
        );
        assert_eq!(
            result,
            (
                replayed_items(vec![("1", "2014-11-28T21:00:00")]),
                Some(parse_dt("2014-12-05T21:00:00"))
            )
        );
    }

    #[test]
    fn resumes_original_schedule_once_caught_up() {
        let items = cached_entries(
            1,
            vec![
                ("1", "2014-11-01T09:00:00", "pub"),
                ("2", "2014-11-04T21:00:00", "pub"),
                ("3", "2014-11-09T22:00:00", "pub"),
                ("4", "2014-11-13T22:00:00", "pub"),
            ],
        );
        let result = reschedule_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-03T20:00:00")),
            parse_dt("2014-11-03T20:00:00"),
            parse_dt("2014-11-12T22:00:00"),
            parse_dt("2014-11-12T22:00:00"),
        );
        assert_eq!(
            result,
            (
                replayed_items(vec![
                    ("1", "2014-11-03T20:00:00"),
                    ("2", "2014-11-04T21:00:00"),
                    ("3", "2014-11-09T22:00:00"),
                ]),
                None
            )
        );
    }

    #[test]
    fn does_not_duplicate_a_rescheduled_item_that_already_played() {
        let items = cached_entries(
            1,
            vec![
                ("1", "2014-11-01T09:00:00", "pub"),
                ("1", "2014-11-04T21:00:00", "pub"),
            ],
        );
        let result = reschedule_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-03T20:00:00")),
            parse_dt("2014-11-03T20:00:00"),
            parse_dt("2014-11-12T22:00:00"),
            parse_dt("2014-11-12T22:00:00"),
        );
        assert_eq!(
            result,
            (replayed_items(vec![("1", "2014-11-03T20:00:00"),]), None)
        );
    }

    #[test]
    fn does_not_schedule_a_replay_if_a_reschedule_is_noticed_before_slot() {
        let items = cached_entries(
            1,
            vec![
                ("1", "2014-11-01T09:00:00", "pub"),
                ("2", "2014-11-02T21:00:00", "pub"),
                ("1", "2014-11-04T21:00:00", "pub"),
            ],
        );
        let result = reschedule_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-06T20:00:00")),
            parse_dt("2014-11-06T20:00:00"),
            parse_dt("2014-12-12T22:00:00"),
            parse_dt("2014-12-12T22:00:00"),
        );
        assert_eq!(
            result,
            (
                replayed_items(vec![
                    ("2", "2014-11-06T20:00:00"),
                    ("1", "2014-11-07T20:00:00"),
                ]),
                None
            )
        );
    }

    #[test]
    fn moved_forward_noticed_after_slot() {
        let items = cached_entries(
            1,
            vec![
                ("1", "2014-11-01T09:00:00", "pub"),
                ("2", "2014-11-02T21:00:00", "pub"),
                ("1", "2014-11-04T21:00:00", "pub"),
            ],
        );
        let result = reschedule_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-03T20:00:00")),
            parse_dt("2014-11-03T20:00:00"),
            parse_dt("2014-12-12T22:00:00"),
            parse_dt("2014-12-12T22:00:00"),
        );
        assert_eq!(
            result,
            (
                replayed_items(vec![
                    ("1", "2014-11-03T20:00:00"),
                    ("2", "2014-11-04T20:00:00"),
                ]),
                None
            )
        );
    }

    #[test]
    fn moved_forward_noticed_before_slot() {
        let items = cached_entries(
            1,
            vec![
                ("1", "2014-11-01T09:00:00", "pub"),
                ("2", "2014-11-02T21:00:00", "pub"),
                ("1", "2014-11-04T21:00:00", "pub"),
            ],
        );
        let result = reschedule_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-06T20:00:00")),
            parse_dt("2014-11-06T20:00:00"),
            parse_dt("2014-12-12T22:00:00"),
            parse_dt("2014-12-12T22:00:00"),
        );
        assert_eq!(
            result,
            (
                replayed_items(vec![
                    ("2", "2014-11-06T20:00:00"),
                    ("1", "2014-11-07T20:00:00"),
                ]),
                None
            )
        );
    }

    #[test]
    fn moved_backward_noticed_before_slot() {
        let items = cached_entries(
            1,
            vec![
                ("1", "2014-11-01T21:00:00", "2014-11-08T21:00:00"),
                ("2", "2014-11-02T09:00:00", "pub"),
                ("1", "2014-11-06T21:00:00", "pub"),
            ],
        );
        let result = reschedule_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-09T20:00:00")),
            parse_dt("2014-11-09T20:00:00"),
            parse_dt("2014-12-12T22:00:00"),
            parse_dt("2014-12-12T22:00:00"),
        );
        assert_eq!(
            result,
            (
                replayed_items(vec![
                    ("1", "2014-11-09T20:00:00"),
                    ("2", "2014-11-10T20:00:00"),
                ]),
                None
            )
        );
    }

    #[test]
    fn moved_backward_noticed_after_slot() {
        let items = cached_entries(
            1,
            vec![
                ("1", "2014-11-01T21:00:00", "2014-11-08T21:00:00"),
                ("2", "2014-11-02T09:00:00", "pub"),
                ("1", "2014-11-06T20:00:00", "pub"),
            ],
        );
        let result = reschedule_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-06T10:00:00")),
            parse_dt("2014-11-06T10:00:00"),
            parse_dt("2014-12-12T22:00:00"),
            parse_dt("2014-11-02T09:00:00"),
        );
        assert_eq!(
            result,
            (
                replayed_items(vec![
                    ("2", "2014-11-06T10:00:00"),
                    ("1", "2014-11-07T10:00:00"),
                ]),
                None
            )
        );
    }

    #[test]
    fn published_retroactively_noticed_before_slot() {
        let items = cached_entries(
            1,
            vec![
                ("1", "2014-11-01T21:00:00", "2014-11-05T21:00:00"),
                ("2", "2014-11-02T09:00:00", "pub"),
            ],
        );
        let result = reschedule_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-06T10:00:00")),
            parse_dt("2014-11-06T10:00:00"),
            parse_dt("2014-12-12T22:00:00"),
            parse_dt("2014-12-12T22:00:00"),
        );
        assert_eq!(
            result,
            (
                replayed_items(vec![
                    ("1", "2014-11-06T10:00:00"),
                    ("2", "2014-11-07T10:00:00"),
                ]),
                None
            )
        );
    }

    #[test]
    fn published_retroactively_noticed_after_slot() {
        let items = cached_entries(
            1,
            vec![
                ("1", "2014-11-01T21:00:00", "2014-11-11T10:00:00"),
                ("2", "2014-11-02T09:00:00", "pub"),
            ],
        );
        let result = reschedule_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-10T20:00:00")),
            parse_dt("2014-11-10T20:00:00"),
            parse_dt("2014-12-12T22:00:00"),
            parse_dt("2014-11-02T09:00:00"),
        );
        assert_eq!(
            result,
            (
                replayed_items(vec![
                    ("2", "2014-11-10T20:00:00"),
                    ("1", "2014-11-11T20:00:00"),
                ]),
                None
            )
        );
    }

    #[test]
    fn published_retroactively_noticed_after_slot_and_missed_a_slot() {
        let items = cached_entries(
            1,
            vec![
                // TODO: OK, so the issue here is that this is the is_first_noticed
                // but it falls AFTER the start of this replay. If we pass in the
                // actual concrete start time I guess we could filter out any
                // noticed before that timestamp while keeping at least one total
                ("1", "2014-11-01T21:00:00", "2014-11-11T21:00:00"),
                ("2", "2014-11-02T09:00:00", "pub"),
            ],
        );
        let result = reschedule_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-10T10:00:00")),
            parse_dt("2014-11-10T10:00:00"),
            parse_dt("2014-12-12T22:00:00"),
            parse_dt("2014-11-02T09:00:00"),
        );
        assert_eq!(
            result,
            (
                replayed_items(vec![
                    ("2", "2014-11-10T10:00:00"),
                    // Because the publish wasn't noticed before the slot on the
                    // 11th, it doesn't get replayed until the next available slot
                    // on the 12th
                    ("1", "2014-11-12T10:00:00"),
                ]),
                None
            )
        );
    }

    #[test]
    fn unpublish_noticed_after_slot() {
        let items = cached_entries(
            1,
            vec![
                ("1", "2014-11-01T21:00:00", "pub"),
                ("2", "2014-11-03T21:00:00", "pub"),
                ("1", "gone", "2014-11-11T21:00:00"),
            ],
        );
        let result = reschedule_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-10T10:00:00")),
            parse_dt("2014-11-10T10:00:00"),
            parse_dt("2014-12-12T22:00:00"),
            parse_dt("2014-12-12T22:00:00"),
        );
        assert_eq!(
            result,
            (
                replayed_items(vec![
                    // We didn't notice the unpublish before the slot was filled, so it
                    // was replayed briefly, but we need to keep the slot open to avoid
                    // shifting anything else around.
                    ("2", "2014-11-11T10:00:00"),
                ]),
                None
            )
        );
    }

    #[test]
    fn unpublish_noticed_before_slot() {
        let items = cached_entries(
            1,
            vec![
                ("1", "2014-11-01T21:00:00", "pub"),
                ("2", "2014-11-03T21:00:00", "pub"),
                ("1", "gone", "2014-11-10T21:00:00"),
            ],
        );
        let result = reschedule_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-12T10:00:00")),
            parse_dt("2014-11-12T10:00:00"),
            parse_dt("2014-12-12T22:00:00"),
            parse_dt("2014-12-12T22:00:00"),
        );
        assert_eq!(
            result,
            (
                replayed_items(vec![
                    // We noticed the unpublish before we got to the slot, so we
                    // publish the next available episode in that slot
                    ("2", "2014-11-12T10:00:00"),
                ]),
                None
            )
        );
    }

    #[test]
    fn items_in_a_retroactively_subscribed_feed_appear_properly() {
        let items = cached_entries(
            1,
            vec![
                ("1", "2014-11-01T21:00:00", "2014-12-20T10:00:00"),
                ("2", "2014-11-03T21:00:00", "2014-12-20T10:00:00"),
                ("3", "2014-11-06T21:00:00", "2014-12-20T10:00:00"),
            ],
        );
        let result = reschedule_feed(
            &items,
            DateRule::daily(parse_dt("2014-11-04T10:00:00")),
            parse_dt("2014-11-04T10:00:00"),
            parse_dt("2014-12-12T22:00:00"),
            parse_dt("2014-12-20T10:00:00"),
        );
        assert_eq!(
            result,
            (
                replayed_items(vec![
                    ("1", "2014-11-04T10:00:00"),
                    ("2", "2014-11-05T10:00:00"),
                    ("3", "2014-11-06T21:00:00"),
                ]),
                None
            )
        );
    }
}
