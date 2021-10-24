use crate::FeedSummaryItem;
use chrono::{DateTime, Utc};
use chronoutil::DateRule;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct ReplayedItem<'a> {
    pub id: &'a str,
    pub timestamp: DateTime<Utc>,
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
