mod utils;

use chrono::{DateTime, Utc};
use chronoutil::DateRule;
use podreplay_lib::{reschedule_feed, FeedSummary};
use wasm_bindgen::prelude::*;

// When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
// allocator.
#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn reschedule(
    feed: &JsValue,
    // rule: DateRule<DateTime<Utc>>,
    start: &JsValue,        // DateTime<Utc>,
    now: &JsValue,          // DateTime<Utc>,
    feed_noticed: &JsValue, // DateTime<Utc>,
) -> JsValue {
    let feed: FeedSummary = feed.into_serde().unwrap();
    let start: DateTime<Utc> = start.into_serde().unwrap();
    let now: DateTime<Utc> = now.into_serde().unwrap();
    let feed_noticed: DateTime<Utc> = feed_noticed.into_serde().unwrap();

    let rule = DateRule::weekly(start);
    let items = feed.into_cached_items();

    let (rescheduled, _) = reschedule_feed(&items, rule, start, now, feed_noticed);
    JsValue::from_serde(&rescheduled).unwrap()
}
