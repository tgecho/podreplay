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
    let mut published_before_cutoff = items
        .iter()
        .filter(|item| item.published.map_or(false, |p| p <= until));
    let mut instances_by_id = create_instances_by_id(items);
    let mut delayed = DelayedItems::new();
    let mut results = Vec::new();

    for slot in rule.with_end(until) {
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
                        if item.noticed > slot {
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
                                results.push(ReplayedItem {
                                    id: &item.id,
                                    timestamp: slot,
                                });
                                instances.already_replayed = true;
                                break; // slot filled, move to the next
                            }
                        }
                    } else if let Some(published) = item.published {
                        // This was published after this slot, meaning we've apparently caught up.
                        // Keep replaying items at their original publication times.
                        results.push(ReplayedItem {
                            id: &item.id,
                            timestamp: published,
                        });
                        instances.already_replayed = true;
                    }
                }
            } else if delayed.is_empty() {
                return results; // ran out of items, don't loop over the rest of the slots
            } else {
                break; // no eligible items available for this slot, try the next
            }
        }
    }
    results
}

fn create_instances_by_id(items: &[FeedSummaryItem]) -> HashMap<&String, Scheduled> {
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
    items: Vec<&'a FeedSummaryItem>,
}

enum Unpublished {
    BeforeSlot,
    AfterSlot,
    Never,
}

impl<'a> Scheduled<'a> {
    fn rescheduled_before<'b>(&'a self, slot: DateTime<Utc>, item: &'b FeedSummaryItem) -> bool {
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
