use crate::CachedEntry;
use chrono::{DateTime, Utc};
use chronoutil::DateRule;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug, PartialEq, Serialize)]
pub struct ReplayedItem {
    pub id: String,
    pub timestamp: DateTime<Utc>,
}

pub type Reschedule = HashMap<String, DateTime<Utc>>;

pub fn replay_feed(
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
