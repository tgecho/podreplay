use chrono::{DateTime, Utc};
use diligent_date_parser::parse_date;
use kuchiki::{parse_html, traits::TendrilSink};
use quick_xml::events::{BytesStart, Event};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, io::BufRead};
use thiserror::Error;

use crate::CachedEntry;

#[derive(Debug)]
struct PartialItem<'a> {
    start: BytesStart<'a>,
    id: Option<String>,
    title: Option<String>,
    timestamp: Option<DateTime<Utc>>,
    had_enclosure: bool,
}

impl<'a> PartialItem<'a> {
    fn complete(self) -> Option<SummaryItem> {
        if self.had_enclosure {
            Some(SummaryItem {
                title: self.title?,
                id: self.id?,
                timestamp: self.timestamp?,
            })
        } else {
            None
        }
    }
}

#[derive(PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct SummaryItem {
    pub id: String,
    pub title: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FeedSummary {
    pub uri: String,
    pub title: String,
    #[serde(skip_serializing)]
    pub marked_private: bool,
    pub items: Vec<SummaryItem>,
}

#[derive(Error, Debug)]
pub enum SummarizeError {
    #[error("Failed to parse feed: {0}")]
    Parse(#[from] quick_xml::Error),
    // #[error("Unexpected")]
    // Parse(#[from] quick_xml::Error),
    #[error("No valid feed found")]
    NotAFeed,
}

impl FeedSummary {
    pub fn new<R: BufRead>(uri: String, reader: &mut R) -> Result<Self, SummarizeError> {
        let reader = quick_xml::Reader::from_reader(reader);
        let (mut items, title, marked_private) = summarize_feed(reader)?;
        items.reverse(); // we're most likely in reverse order
        items.sort_unstable_by_key(|i| i.timestamp); // just to be safe
        Ok(FeedSummary {
            uri,
            title: title.unwrap_or_default(),
            marked_private,
            items,
        })
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

    pub fn len(&self) -> usize {
        self.items.len()
    }
}

pub fn summarize_feed<R: BufRead>(
    mut reader: quick_xml::Reader<R>,
) -> Result<(Vec<SummaryItem>, Option<String>, bool), SummarizeError> {
    let mut results: Vec<SummaryItem> = Vec::new();
    let mut buf: Vec<u8> = Vec::new();
    let mut partial_item: Option<PartialItem> = None;
    let mut xml_decl_found = false;
    let mut feed_title = None;
    let mut marked_private = false;

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
                        had_enclosure: false,
                    });
                }
                b"guid" | b"id" => {
                    if let Some(item) = &mut partial_item {
                        item.id = Some(read_contents(&mut reader, &start)?);
                    }
                }
                b"title" => {
                    if let Ok(title) = read_contents(&mut reader, &start) {
                        if let Some(item) = &mut partial_item {
                            item.title = Some(title);
                        } else {
                            feed_title = Some(title);
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
                            .and_then(|s| parse_timestamp(&s))
                        {
                            item.timestamp = Some(timestamp);
                        }
                    }
                }
                b"itunes:block" if partial_item.is_none() => {
                    let name = start.name().to_owned();
                    if let Ok(block) = reader.read_text(name, &mut buf) {
                        marked_private = block.to_ascii_lowercase() == "yes"
                    }
                }
                b"enclosure" | b"link" => {
                    if is_audio_enclosure(&start) {
                        if let Some(item) = &mut partial_item {
                            item.had_enclosure = true;
                        }
                    }
                }
                _ => {}
            },
            Ok(Event::Empty(empty)) => match empty.name() {
                b"enclosure" | b"link" if is_audio_enclosure(&empty) => {
                    if let Some(item) = &mut partial_item {
                        item.had_enclosure = true;
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
        Ok((results, feed_title, marked_private))
    }
}

pub fn is_audio_enclosure(start: &BytesStart) -> bool {
    let mut rel_enclosure = false;
    let mut type_audio = false;
    for attr in start.attributes().filter_map(|a| a.ok()) {
        if attr.key == b"rel" {
            rel_enclosure = attr.value.starts_with(b"enclosure");
        } else if attr.key == b"type" {
            type_audio = attr.value.starts_with(b"audio/");
        }
    }
    match start.name() {
        b"enclosure" => type_audio,
        b"link" => type_audio && rel_enclosure,
        _ => false,
    }
}

pub fn read_contents<R: BufRead>(
    reader: &mut quick_xml::Reader<R>,
    start: &BytesStart,
) -> Result<String, quick_xml::Error> {
    let mut id_buf: Vec<u8> = Vec::new();
    let mut id = String::new();
    loop {
        match reader.read_event(&mut id_buf)? {
            Event::Text(bytes) => {
                if let Ok(frag) = bytes.unescape_and_decode(reader) {
                    id.push_str(frag.trim());
                }
            }
            Event::CData(bytes) => {
                if let Ok(frag) = bytes.partial_escape().unescape_and_decode(reader) {
                    id.push_str(frag.trim());
                }
            }
            Event::End(end) if end.name() == start.name() => break,
            Event::Eof => {
                return Err(quick_xml::Error::UnexpectedEof(format!(
                    "while attempting to get {:?}",
                    start.name()
                )))
            }
            _ => return Err(quick_xml::Error::TextNotFound),
        }
    }
    if id.is_empty() {
        Err(quick_xml::Error::TextNotFound)
    } else {
        Ok(id.to_string())
    }
}

pub fn parse_timestamp(timestamp_str: &str) -> Option<DateTime<Utc>> {
    parse_date(timestamp_str).map(|ts| ts.into())
}

fn html_to_text(html: &str) -> String {
    let node = parse_html().one(html);
    node.text_contents()
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

    #[test]
    fn megaphone() {
        let xml = include_str!("../tests/data/megaphone.xml");
        let output = FeedSummary::new("testing".into(), &mut Cursor::new(xml)).unwrap();
        let expected = vec![
            SummaryItem {
                id: "612990fc-4f9c-11eb-a6af-e7830eb4fc55".to_string(),
                title: "S6 Ep. 6: No Peace".to_string(),
                timestamp: parse_dt("2021-12-15T08:00:00"),
            },
            SummaryItem {
                id: "613b2312-4f9c-11eb-a6af-b700e1b799da".to_string(),
                title: "S6 Ep. 7: Into Ashes".to_string(),
                timestamp: parse_dt("2021-12-22T08:00:00"),
            },
            SummaryItem {
                id: "614f5f12-4f9c-11eb-a6af-cb9557e04485".to_string(),
                title: "S6 Ep. 8: Damages".to_string(),
                timestamp: parse_dt("2021-12-29T08:00:00"),
            },
        ];
        assert_eq!(output.items, expected);
    }
}
