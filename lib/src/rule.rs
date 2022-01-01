use std::collections::HashSet;

use chrono::{DateTime, Datelike, Duration, Utc, Weekday};
use chronoutil::DateRule;
use itertools::Itertools;
use nom::IResult;

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

impl Rule {
    fn parse(start: DateTime<Utc>, s: &str) -> Result<Rule, String> {
        use nom::{character::complete::one_of, sequence::tuple};
        tuple((interval, one_of("mwd"), weekdays))(s)
            .map(|(_, (interval, freq, days))| match freq {
                'm' => Rule::Monthly { start, interval },
                'w' => Rule::Weekly {
                    start,
                    interval,
                    days,
                },
                'd' => Rule::Daily { start, interval },
                _ => panic!("freq parser returned an invalid tag"),
            })
            .map_err(|err| err.to_string())
    }
}

fn interval(s: &str) -> IResult<&str, usize> {
    use nom::{
        character::complete::digit1,
        combinator::{map_res, verify},
    };
    verify(map_res(digit1, |d: &str| d.parse()), |n| n > &0)(s)
}

fn weekdays(s: &str) -> IResult<&str, HashSet<Weekday>> {
    use nom::{
        bytes::complete::tag,
        combinator::{map, opt},
        sequence::tuple,
    };

    let parsed = tuple((
        opt(map(tag("Su"), |_| Weekday::Sun)),
        opt(map(tag("M"), |_| Weekday::Mon)),
        opt(map(tag("Tu"), |_| Weekday::Tue)),
        opt(map(tag("W"), |_| Weekday::Wed)),
        opt(map(tag("Th"), |_| Weekday::Thu)),
        opt(map(tag("F"), |_| Weekday::Fri)),
        opt(map(tag("Sa"), |_| Weekday::Sat)),
    ))(s);
    parsed.map(|(remaining, (su, m, tu, w, th, f, sa))| {
        (
            remaining,
            [su, m, tu, w, th, f, sa].into_iter().flatten().collect(),
        )
    })
}

pub fn parse_rule(start: DateTime<Utc>, rule_str: &str) -> Rule {
    Rule::parse(start, rule_str).unwrap_or_else(|_| Rule::Weekly {
        start,
        interval: 1,
        days: HashSet::new(),
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
