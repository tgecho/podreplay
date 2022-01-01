use std::collections::HashSet;

use chrono::{DateTime, Datelike, Duration, Utc, Weekday};
use chronoutil::DateRule;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

pub enum Rule {
    Monthly {
        start: DateTime<Utc>,
        interval: usize,
    },
    Weekly {
        start: DateTime<Utc>,
        interval: usize,
        days: HashSet<Weekday>,
    },
    Daily {
        start: DateTime<Utc>,
        interval: usize,
    },
}

impl IntoIterator for Rule {
    type Item = DateTime<Utc>;
    type IntoIter = Box<dyn Iterator<Item = DateTime<Utc>>>;

    fn into_iter(self) -> Self::IntoIter {
        match self {
            Rule::Monthly { start, interval } => {
                Box::new(DateRule::monthly(start).step_by(interval))
            }
            Rule::Daily { start, interval } => Box::new(DateRule::daily(start).step_by(interval)),
            Rule::Weekly {
                start,
                interval,
                days,
            } => {
                if days.is_empty() {
                    Box::new(DateRule::weekly(start).step_by(interval))
                } else {
                    let iter = days
                        .into_iter()
                        .map(|weekday| {
                            let offset = days_until_weekday(start, weekday);
                            let rule = DateRule::weekly(start + Duration::days(offset as i64));
                            rule.step_by(interval)
                        })
                        .kmerge();
                    Box::new(iter)
                }
            }
        }
    }
}

pub fn parse_rule(start: DateTime<Utc>, rule_str: &str) -> Rule {
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r"^(?P<interval>\d+)(?P<freq>m|w|d)(?P<Su>Su)?(?P<M>M)?(?P<Tu>Tu)?(?P<W>W)?(?P<Th>Th)?(?P<F>F)?(?P<Sa>Sa)?$")
                .unwrap();
    }
    if let Some(capture) = RE.captures(rule_str) {
        let interval = capture
            .name("interval")
            .map(|m| {
                m.as_str()
                    .parse::<usize>()
                    .expect("regex \\d captured a non-integer!")
            })
            .unwrap_or(1);

        match capture.name("freq").map(|m| m.as_str()) {
            Some("m") => Rule::Monthly { start, interval },
            Some("d") => Rule::Daily { start, interval },
            _ => {
                let days: HashSet<_> = ["Su", "M", "Tu", "W", "Th", "F", "Sa"]
                    .iter()
                    .filter_map(|name| {
                        let day = capture.name(name)?.as_str();
                        abbr_to_weekday(day)
                    })
                    .collect();
                Rule::Weekly {
                    start,
                    interval,
                    days,
                }
            }
        }
    } else {
        Rule::Weekly {
            start,
            interval: 1,
            days: HashSet::new(),
        }
    }
}

fn abbr_to_weekday(abbr: &str) -> Option<Weekday> {
    Some(match abbr {
        "Su" => Weekday::Sun,
        "M" => Weekday::Mon,
        "Tu" => Weekday::Tue,
        "W" => Weekday::Wed,
        "Th" => Weekday::Thu,
        "F" => Weekday::Fri,
        "Sa" => Weekday::Sat,
        _ => return None,
    })
}

fn days_until_weekday<D: Datelike>(date: D, weekday: Weekday) -> u32 {
    let base = date.weekday().num_days_from_sunday();
    let target = weekday.num_days_from_sunday();
    (7 + target - base) % 7
}

#[cfg(test)]
mod test {
    use chrono::Weekday;

    use crate::{
        rule::{days_until_weekday, parse_rule},
        test_helpers::parse_dt,
    };

    #[test]
    fn test_days_until_weekday() {
        let date = parse_dt("2013-10-10T21:00:00");
        assert_eq!(days_until_weekday(date, Weekday::Thu), 0);
        assert_eq!(days_until_weekday(date, Weekday::Fri), 1);
        assert_eq!(days_until_weekday(date, Weekday::Sat), 2);
        assert_eq!(days_until_weekday(date, Weekday::Sun), 3);
        assert_eq!(days_until_weekday(date, Weekday::Mon), 4);
        assert_eq!(days_until_weekday(date, Weekday::Tue), 5);
        assert_eq!(days_until_weekday(date, Weekday::Wed), 6);
    }

    #[test]
    fn test_parse_daily() {
        assert_eq!(
            parse_rule(parse_dt("2013-10-10T21:00:00"), "1d")
                .into_iter()
                .take(3)
                .collect::<Vec<_>>(),
            vec![
                parse_dt("2013-10-10T21:00:00"),
                parse_dt("2013-10-11T21:00:00"),
                parse_dt("2013-10-12T21:00:00"),
            ]
        );
    }

    #[test]
    fn test_parse_every_3_days() {
        assert_eq!(
            parse_rule(parse_dt("2013-10-10T21:00:00"), "3d")
                .into_iter()
                .take(3)
                .collect::<Vec<_>>(),
            vec![
                parse_dt("2013-10-10T21:00:00"),
                parse_dt("2013-10-13T21:00:00"),
                parse_dt("2013-10-16T21:00:00"),
            ]
        );
    }

    #[test]
    fn test_parse_monthly() {
        assert_eq!(
            parse_rule(parse_dt("2013-10-10T21:00:00"), "1m")
                .into_iter()
                .take(3)
                .collect::<Vec<_>>(),
            vec![
                parse_dt("2013-10-10T21:00:00"),
                parse_dt("2013-11-10T21:00:00"),
                parse_dt("2013-12-10T21:00:00"),
            ]
        );
    }

    #[test]
    fn test_parse_every_2_months() {
        assert_eq!(
            parse_rule(parse_dt("2013-10-10T21:00:00"), "2m")
                .into_iter()
                .take(3)
                .collect::<Vec<_>>(),
            vec![
                parse_dt("2013-10-10T21:00:00"),
                parse_dt("2013-12-10T21:00:00"),
                parse_dt("2014-02-10T21:00:00"),
            ]
        );
    }

    #[test]
    fn test_parse_weekly() {
        assert_eq!(
            parse_rule(parse_dt("2013-10-10T21:00:00"), "1w")
                .into_iter()
                .take(3)
                .collect::<Vec<_>>(),
            vec![
                parse_dt("2013-10-10T21:00:00"),
                parse_dt("2013-10-17T21:00:00"),
                parse_dt("2013-10-24T21:00:00"),
            ]
        );
    }

    #[test]
    fn test_parse_every_3_weeks() {
        assert_eq!(
            parse_rule(parse_dt("2013-10-10T21:00:00"), "3w")
                .into_iter()
                .take(3)
                .collect::<Vec<_>>(),
            vec![
                parse_dt("2013-10-10T21:00:00"),
                parse_dt("2013-10-31T21:00:00"),
                parse_dt("2013-11-21T21:00:00"),
            ]
        );
    }

    #[test]
    fn test_parse_weekly_every_other_friday() {
        assert_eq!(
            parse_rule(parse_dt("2013-10-10T21:00:00"), "2wF")
                .into_iter()
                .take(3)
                .collect::<Vec<_>>(),
            vec![
                parse_dt("2013-10-11T21:00:00"),
                parse_dt("2013-10-25T21:00:00"),
                parse_dt("2013-11-08T21:00:00"),
            ]
        );
    }

    #[test]
    fn test_parse_weekly_every_tue_and_sat() {
        assert_eq!(
            parse_rule(parse_dt("2013-10-10T21:00:00"), "1wTuSa")
                .into_iter()
                .take(3)
                .collect::<Vec<_>>(),
            vec![
                parse_dt("2013-10-12T21:00:00"),
                parse_dt("2013-10-15T21:00:00"),
                parse_dt("2013-10-19T21:00:00"),
            ]
        );
    }
}
