use chronoutil::DateRule;

mod helpers;
use helpers::{cached_entries, parse_dt};

use podreplay_lib::{replay_feed, ReplayedItem};

fn replayed_items<'a>(items: Vec<(&'a str, &'a str)>) -> Vec<ReplayedItem> {
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
    let items = cached_entries(1, vec![]);
    let result = replay_feed(
        &items,
        DateRule::daily(parse_dt("2014-11-28T21:00:00")),
        parse_dt("2014-11-28T21:00:00"),
        parse_dt("2014-12-28T21:00:00"),
        parse_dt("2014-12-28T21:00:00"),
    );
    assert_eq!(result, (vec![], None));
}

#[test]
fn one_item() {
    let items = cached_entries(1, vec![("1", "2013-10-10T21:00:00", "pub")]);
    let result = replay_feed(
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
    let result = replay_feed(
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
    let result = replay_feed(
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
    let result = replay_feed(
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
    let result = replay_feed(
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
    let result = replay_feed(
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
    let result = replay_feed(
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
    let result = replay_feed(
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
    let result = replay_feed(
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
    let result = replay_feed(
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
    let result = replay_feed(
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
    let result = replay_feed(
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
    let result = replay_feed(
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
    let result = replay_feed(
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
    let result = replay_feed(
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
    let result = replay_feed(
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
