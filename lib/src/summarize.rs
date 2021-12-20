use chrono::{DateTime, Utc};
use diligent_date_parser::parse_date;
use quick_xml::events::{BytesStart, Event};
use std::{collections::HashMap, io::BufRead};
use thiserror::Error;

#[derive(Debug)]
struct PartialItem<'a> {
    start: BytesStart<'a>,
    id: Option<String>,
    timestamp: Option<DateTime<Utc>>,
}

impl<'a> PartialItem<'a> {
    fn complete(self) -> Option<Item> {
        Some(Item {
            id: self.id?,
            timestamp: self.timestamp?,
        })
    }
}

#[derive(PartialEq, Debug)]
pub struct Item {
    pub id: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug)]
pub struct FeedSummary {
    items: Vec<Item>,
}

#[derive(Error, Debug)]
pub enum SummarizeError {
    #[error("failed to parse feed")]
    Parse(#[from] quick_xml::Error),
}

impl FeedSummary {
    pub fn new<R: BufRead>(reader: &mut R) -> Result<Self, SummarizeError> {
        let reader = quick_xml::Reader::from_reader(reader);
        summarize_feed(reader)
    }

    pub fn from_items(items: Vec<Item>) -> Self {
        FeedSummary { items }
    }

    pub fn id_map(&self) -> HashMap<&str, &Item> {
        self.items.iter().map(|e| (e.id.as_str(), e)).collect()
    }
}

pub fn summarize_feed<R: BufRead>(
    mut reader: quick_xml::Reader<R>,
) -> Result<FeedSummary, SummarizeError> {
    let mut results: Vec<Item> = Vec::new();
    let mut buf: Vec<u8> = Vec::new();
    let mut partial_item: Option<PartialItem> = None;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(start)) => match start.name() {
                b"item" | b"entry" => {
                    partial_item = Some(PartialItem {
                        start: start.to_owned(),
                        id: None,
                        timestamp: None,
                    });
                }
                b"guid" | b"id" => {
                    if let Some(item) = &mut partial_item {
                        let name = start.name().to_owned();
                        if let Ok(id) = reader.read_text(name, &mut buf) {
                            item.id = Some(id);
                        }
                    }
                }
                b"pubDate" | b"updated" => {
                    if let Some(item) = &mut partial_item {
                        let name = start.name().to_owned();
                        if let Some(timestamp) = reader
                            .read_text(&name, &mut buf)
                            .ok()
                            .and_then(parse_timestamp)
                        {
                            item.timestamp = Some(timestamp);
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::End(end)) => {
                if let Some(item) = &partial_item {
                    if item.start.name() == end.name() {
                        if let Some(complete) = partial_item.take().and_then(|i| i.complete()) {
                            results.push(complete);
                        }
                    }
                }
            }
            Ok(_) => {
                continue;
            }
            Err(e) => {
                tracing::error!("Error at position {}: {:?}", reader.buffer_position(), e);
                return Err(e.into());
            }
        }
        buf.clear();
    }
    Ok(FeedSummary::from_items(results))
}

fn parse_timestamp(timestamp_str: String) -> Option<DateTime<Utc>> {
    parse_date(&timestamp_str).map(|ts| ts.into())
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::{FeedSummary, Item};
    use crate::test_helpers::parse_dt;
    use pretty_assertions::assert_eq;

    #[test]
    fn atom() {
        let xml = include_str!("../tests/data/sample_atom.xml");
        let output = FeedSummary::new(&mut Cursor::new(xml)).unwrap();
        let expected = vec![Item {
            id: "urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a".to_string(),
            timestamp: parse_dt("2003-12-13T18:30:02"),
        }];
        assert_eq!(output.items, expected);
    }

    #[test]
    fn rss2() {
        let xml = include_str!("../tests/data/sample_rss_2.0.xml");
        let output = FeedSummary::new(&mut Cursor::new(xml)).unwrap();
        let expected = vec![
            Item {
                id: "http://scriptingnews.userland.com/backissues/2002/09/29#When:6:56:02PM"
                    .to_string(),
                timestamp: parse_dt("2002-09-30T01:56:02"),
            },
            Item {
                id: "http://scriptingnews.userland.com/backissues/2002/09/29#When:12:59:01PM"
                    .to_string(),
                timestamp: parse_dt("2002-09-29T19:59:01"),
            },
        ];
        assert_eq!(output.items, expected);
    }
}
