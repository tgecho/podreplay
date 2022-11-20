#![allow(clippy::unused_unit)] // https://github.com/rustwasm/wasm-bindgen/issues/2774

#[cfg(debug_assertions)]
mod utils;

use chrono::{DateTime, TimeZone, Utc};
use podreplay_lib::{parse_rule, reschedule_feed, Item};
use wasm_bindgen::prelude::*;

fn dt_from_unix_epoch(seconds: f64) -> DateTime<Utc> {
    Utc.timestamp_opt(seconds as i64, 0).unwrap()
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
    rule: &str,
    start: f64,
    first: Option<f64>,
    last: Option<f64>,
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
    let rule = parse_rule(start, rule);
    let first = first.map(dt_from_unix_epoch);
    let last = last.map(dt_from_unix_epoch);

    let (rescheduled, _) = reschedule_feed(&items, rule, start, None, None, first, last);

    (0..length)
        .into_iter()
        .map(|index| {
            let timestamp = rescheduled.get(&index);
            timestamp.map_or(0.0, |ts| ts.timestamp() as f64)
        })
        .collect()
}
