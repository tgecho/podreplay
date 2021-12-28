mod utils;

use chrono::{DateTime, Utc};
use chronoutil::DateRule;
use podreplay_lib::{reschedule_feed, Item};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

use std::time::{Duration, UNIX_EPOCH};

fn dt_from_unix_epoch(seconds: f64) -> DateTime<Utc> {
    (UNIX_EPOCH + Duration::from_secs_f64(seconds)).into()
}

struct TinyItem {
    id: usize,
    timestamp: DateTime<Utc>,
}

impl Item<usize> for TinyItem {
    fn id(&self) -> &usize {
        &self.id
    }

    fn published(&self) -> Option<DateTime<Utc>> {
        Some(self.timestamp)
    }

    fn noticed(&self) -> DateTime<Utc> {
        self.timestamp
    }
}

#[wasm_bindgen]
pub fn reschedule(
    timestamps: &[f64],
    // rule: &str,
    start: f64,
) -> Vec<f64> {
    #[cfg(debug_assertions)]
    utils::set_panic_hook();

    let length = timestamps.len();
    let items: Vec<_> = timestamps
        .iter()
        .enumerate()
        .map(|(index, timestamp)| TinyItem {
            id: index,
            timestamp: dt_from_unix_epoch(*timestamp),
        })
        .collect();
    let start = dt_from_unix_epoch(start);

    // TODO: don't hardcode this (we need to add a way to configure this on the /replay endpoint as well)
    let rule = DateRule::weekly(start);

    let (rescheduled, _) = reschedule_feed(&items, rule, start, None, None);

    (0..length)
        .into_iter()
        .map(|index| {
            let timestamp = rescheduled.get(&index);
            timestamp.map_or(0.0, |ts| ts.timestamp() as f64)
        })
        .collect()
}
