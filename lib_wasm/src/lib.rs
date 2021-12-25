mod utils;

use chrono::{DateTime, Utc};
use chronoutil::DateRule;
use podreplay_lib::{reschedule_feed, FeedSummary, SummaryItem};
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

#[wasm_bindgen]
pub fn reschedule(
    items: Vec<f64>,
    // rule: &str,
    start: f64,
    now: f64,
    feed_noticed: f64,
) -> Vec<f64> {
    let length = items.len();
    let feed = FeedSummary::from_items(
        items
            .iter()
            .enumerate()
            .map(|(index, timestamp)| SummaryItem {
                id: index.to_string(),
                title: "".to_string(),
                timestamp: dt_from_unix_epoch(*timestamp),
            })
            .collect(),
    );
    let start = dt_from_unix_epoch(start);
    let now = dt_from_unix_epoch(now);
    let feed_noticed = dt_from_unix_epoch(feed_noticed);

    // TODO: don't hardcode this (we need to add a way to configure this on the /replay endpoint as well)
    let rule = DateRule::weekly(start);
    let items = feed.into_cached_items();

    let (rescheduled, _) = reschedule_feed(&items, rule, start, now, feed_noticed);

    (0..length)
        .into_iter()
        .map(|index| {
            let timestamp = rescheduled.get(&index.to_string());
            timestamp.map_or(0.0, |ts| ts.timestamp() as f64)
        })
        .collect()
}
