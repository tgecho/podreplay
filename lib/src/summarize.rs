use chrono::{DateTime, Utc};
use diligent_date_parser::parse_date;
use kuchiki::{parse_html, traits::TendrilSink};
use quick_xml::events::{BytesStart, Event};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::BufRead};
use thiserror::Error;
use url::Url;

use crate::CachedEntry;

#[derive(Debug)]
struct PartialItem<'a> {
    start: BytesStart<'a>,
    id: Option<String>,
    title: Option<String>,
    timestamp: Option<DateTime<Utc>>,
}

impl<'a> PartialItem<'a> {
    fn complete(self) -> Option<SummaryItem> {
        Some(SummaryItem {
            title: self.title?,
            id: self.id?,
            timestamp: self.timestamp?,
        })
    }
}

#[derive(PartialEq, Debug, Serialize, Deserialize)]
pub struct SummaryItem {
    pub id: String,
    pub title: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedSummary {
    uri: String,
    items: Vec<SummaryItem>,
}

#[derive(Error, Debug)]
pub enum SummarizeError {
    #[error("failed to parse feed")]
    Parse(#[from] quick_xml::Error),
    #[error("no valid feed found")]
    NotAFeed,
}

impl FeedSummary {
    pub fn new<R: BufRead>(uri: String, reader: &mut R) -> Result<Self, SummarizeError> {
        let reader = quick_xml::Reader::from_reader(reader);
        let items = summarize_feed(reader)?;
        Ok(FeedSummary::from_items(uri, items))
    }

    pub fn from_items(uri: String, mut items: Vec<SummaryItem>) -> Self {
        items.reverse(); // we're most likely in reverse order
        items.sort_unstable_by_key(|i| i.timestamp); // just to be safe
        FeedSummary { uri, items }
    }

    pub fn id_map(&self) -> HashMap<&str, &SummaryItem> {
        self.items.iter().map(|e| (e.id.as_str(), e)).collect()
    }

    pub fn into_cached_items(self) -> Vec<CachedEntry> {
        self.items
            .into_iter()
            .map(|i| CachedEntry {
                id: i.id,
                feed_id: 0,
                noticed: i.timestamp,
                published: Some(i.timestamp),
            })
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

pub fn summarize_feed<R: BufRead>(
    mut reader: quick_xml::Reader<R>,
) -> Result<Vec<SummaryItem>, SummarizeError> {
    let mut results: Vec<SummaryItem> = Vec::new();
    let mut buf: Vec<u8> = Vec::new();
    let mut partial_item: Option<PartialItem> = None;
    let mut xml_decl_found = false;

    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Decl(_)) => {
                xml_decl_found = true;
            }
            Ok(Event::Eof) => break,
            Ok(Event::Start(start)) => match start.name() {
                b"item" | b"entry" => {
                    partial_item = Some(PartialItem {
                        start: start.to_owned(),
                        id: None,
                        title: None,
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
                b"title" => {
                    if let Some(item) = &mut partial_item {
                        let name = start.name().to_owned();
                        if let Ok(title) = reader.read_text(name, &mut buf) {
                            item.title = Some(title);
                        }
                    }
                }
                b"description" => {
                    if let Some(item) = &mut partial_item {
                        if item.title.is_none() {
                            let name = start.name().to_owned();
                            if let Ok(description) = reader.read_text(name, &mut buf) {
                                let text = html_to_text(&description);
                                let title = if text.len() > 100 {
                                    format!("{}...", text.chars().take(90).collect::<String>())
                                } else {
                                    text
                                };
                                item.title = Some(title);
                            }
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
    if results.is_empty() && !xml_decl_found {
        Err(SummarizeError::NotAFeed)
    } else {
        Ok(results)
    }
}

fn parse_timestamp(timestamp_str: String) -> Option<DateTime<Utc>> {
    parse_date(&timestamp_str).map(|ts| ts.into())
}

fn html_to_text(html: &str) -> String {
    let node = parse_html().one(html);
    node.text_contents()
}

pub fn find_feed_links<R: BufRead>(reader: &mut R, origin: &str) -> Vec<String> {
    parse_html()
        .from_utf8()
        .read_from(reader)
        .ok()
        .and_then(|doc| {
            let base_url = doc
                .select_first("base")
                .ok()
                .and_then(|base| {
                    let attr = base.attributes.borrow();
                    let href = attr.get("href")?;
                    Url::parse(href).ok()
                })
                .or_else(|| Url::parse(origin).ok());
            let base = Url::options().base_url(base_url.as_ref());
            Some(
                doc.select("link[rel=\"alternate\"]")
                    .ok()?
                    .filter_map(|el| {
                        let attr = el.attributes.borrow();
                        let is_a_feed = attr.get("type").map_or(false, |t| {
                            t == "application/rss+xml" || t == "application/atom+xml"
                        });
                        if is_a_feed {
                            attr.get("href").and_then(|href| {
                                let url = base.parse(href).ok()?;
                                Some(url.to_string())
                            })
                        } else {
                            None
                        }
                    })
                    .collect(),
            )
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod test {
    use std::io::Cursor;

    use super::{FeedSummary, SummaryItem};
    use crate::test_helpers::parse_dt;
    use pretty_assertions::assert_eq;

    #[test]
    fn atom() {
        let xml = include_str!("../tests/data/sample_atom.xml");
        let output = FeedSummary::new("testing".into(), &mut Cursor::new(xml)).unwrap();
        let expected = vec![SummaryItem {
            id: "urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a".to_string(),
            title: "Atom-Powered Robots Run Amok".to_string(),
            timestamp: parse_dt("2003-12-13T18:30:02"),
        }];
        assert_eq!(output.items, expected);
    }

    #[test]
    fn rss2() {
        let xml = include_str!("../tests/data/sample_rss_2.0.xml");
        let output = FeedSummary::new("testing".into(), &mut Cursor::new(xml)).unwrap();
        let expected = vec![
            SummaryItem {
                id: "http://scriptingnews.userland.com/backissues/2002/09/29#When:12:59:01PM"
                    .to_string(),
                title: "Joshua Allen: Who loves namespaces?".to_string(),
                timestamp: parse_dt("2002-09-29T19:59:01"),
            },
            SummaryItem {
                id: "http://scriptingnews.userland.com/backissues/2002/09/29#When:6:56:02PM"
                    .to_string(),
                title: "With any luck we should have one or two more days of namespaces stuff here on Scripting Ne..."
                    .to_string(),
                timestamp: parse_dt("2002-09-30T01:56:02"),
            },
        ];
        assert_eq!(output.items, expected);
    }

    mod feed_links {
        use crate::find_feed_links;
        use std::io::Cursor;

        #[test]
        fn test_empty() {
            let actual = find_feed_links(&mut Cursor::new("".to_string()), "");
            let expected: Vec<String> = Vec::new();
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_absolute() {
            let html = r#"
                <link rel="alternate" type="application/rss+xml" href="http://example.com/feed">
            "#;
            let actual = find_feed_links(&mut Cursor::new(html.to_string()), "http://example.com");
            let expected: Vec<String> = vec!["http://example.com/feed".to_string()];
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_relative_with_origin() {
            let html = r#"
                <link rel="alternate" type="application/rss+xml" href="/feed">
            "#;
            let actual = find_feed_links(&mut Cursor::new(html.to_string()), "http://example.com");
            let expected: Vec<String> = vec!["http://example.com/feed".to_string()];
            assert_eq!(actual, expected);
        }

        #[test]
        fn test_relative_with_base() {
            let html = r#"
                <base href="http://example.org/">
                <link rel="alternate" type="application/atom+xml" href="/feed">
            "#;
            let actual = find_feed_links(&mut Cursor::new(html.to_string()), "http://example.com");
            let expected: Vec<String> = vec!["http://example.org/feed".to_string()];
            assert_eq!(actual, expected);
        }
    }
}
