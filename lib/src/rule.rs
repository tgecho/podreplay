use chrono::{DateTime, Datelike, Duration, Utc, Weekday};
use chronoutil::DateRule;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;

pub type RuleIter = Box<dyn Iterator<Item = DateTime<Utc>>>;

pub fn parse_rule(start: DateTime<Utc>, rule_str: &str) -> RuleIter {
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
            Some("m") => Box::new(DateRule::monthly(start).step_by(interval)),
            Some("d") => Box::new(DateRule::daily(start).step_by(interval)),
            _ => {
                let rules: Vec<_> = ["Su", "M", "Tu", "W", "Th", "F", "Sa"]
                    .iter()
                    .filter(|n| capture.name(n).is_some())
                    .filter_map(|day| {
                        let weekday = abbr_to_weekday(day)?;
                        let offset = days_until_weekday(start, weekday);
                        let rule = DateRule::weekly(start + Duration::days(offset as i64));
                        Some(rule.step_by(interval))
                    })
                    .collect();

                if rules.is_empty() {
                    Box::new(DateRule::weekly(start).step_by(interval))
                } else {
                    Box::new(rules.into_iter().kmerge())
                }
            }
        }
    } else {
        Box::new(DateRule::weekly(start))
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
