use crate::reschedule::Reschedule;
use crate::summarize::{is_audio_enclosure, read_contents};
use chrono::{DateTime, SecondsFormat, Utc};
use quick_xml::events::{BytesStart, BytesText, Event};
use quick_xml::name::QName;
use quick_xml::{Reader, Writer};
use std::io::{BufRead, Write};
use thiserror::Error;

// TODO: Consider how/if we can rewrite (or omit?) the
// pubDate/lastPubDate/ttl/skipHours/skipDays channel elements

#[derive(Error, Debug)]
pub enum RewriteError {
    #[error("Failed to parse feed: {0}")]
    Parse(#[from] quick_xml::Error),
    #[error("Failed to write feed")]
    Write(quick_xml::Error),
}

pub fn rewrite_feed(
    xml: &[u8],
    reschedule: &Reschedule<String>,
    pretty: bool,
    mark_as_private: bool,
    custom_title: &Option<String>,
) -> Result<Vec<u8>, RewriteError> {
    let reader = quick_xml::Reader::from_reader(xml);
    let mut output = Vec::new();
    let writer = if pretty {
        quick_xml::Writer::new_with_indent(&mut output, b' ', 4)
    } else {
        quick_xml::Writer::new(&mut output)
    };
    rewrite_feed_to_writer(reader, writer, reschedule, mark_as_private, custom_title)?;
    Ok(output)
}

fn write_itunes_block<W: Write>(writer: &mut Writer<W>) -> Result<(), RewriteError> {
    for ev in element(BytesStart::new("itunes:block"), "Yes".into()) {
        writer.write_event(ev)?;
    }
    Ok(())
}

fn rewrite_feed_to_writer<W: Write>(
    mut reader: quick_xml::Reader<&[u8]>,
    mut writer: quick_xml::Writer<W>,
    reschedule: &Reschedule<String>,
    mark_as_private: bool,
    custom_title: &Option<String>,
) -> Result<(), RewriteError> {
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(start)) => match start.name() {
                QName(b"item") | QName(b"entry") => {
                    rewrite_or_skip_item(start, &mut reader, &mut writer, reschedule)?;
                }
                QName(b"channel") if mark_as_private => {
                    writer.write_event(Event::Start(start))?;
                    write_itunes_block(&mut writer)?;
                }
                QName(b"feed") if mark_as_private => {
                    let is_atom = start.attributes().filter_map(|a| a.ok()).any(|a| {
                        a.key == QName(b"xmlns")
                            && a.value.as_ref() == b"http://www.w3.org/2005/Atom"
                    });
                    writer.write_event(Event::Start(start.clone()))?;
                    if is_atom {
                        write_itunes_block(&mut writer)?;
                    }
                }
                QName(b"title") => {
                    let existing_title = reader.read_text(start.name()).ok();
                    let title = custom_title
                        .clone()
                        .or_else(|| existing_title.map(|title| format!("{title} (PodReplay)")))
                        .unwrap_or_else(|| "Untitled Podreplay Feed".to_string());
                    for ev in element(start, title) {
                        writer.write_event(ev)?;
                    }
                }
                _ => {
                    writer.write_event(Event::Start(start))?;
                }
            },
            Ok(ev) => {
                writer.write_event(ev).map_err(RewriteError::Write)?;
            }
            Err(e) => {
                tracing::error!("Error at position {}: {:?}", reader.buffer_position(), e);
                return Err(RewriteError::Parse(e));
            }
        }
        buf.clear();
    }
    Ok(())
}
//hello
fn rewrite_or_skip_item<B: BufRead, W: Write>(
    start: BytesStart,
    reader: &mut Reader<B>,
    writer: &mut Writer<W>,
    reschedule: &Reschedule<String>,
) -> Result<(), quick_xml::Error> {
    let item_tag = start.name();
    let mut buf = Vec::new();
    let mut events = Vec::new();
    let mut skipped_timestamp: Option<(usize, BytesStart)> = None;
    let mut target_timestamp: Option<DateTime<Utc>> = None;
    let mut had_id = false;
    let mut had_timestamp = false;
    let mut had_enclosure = false;
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::End(end)) if end.name() == item_tag => {
                // We can't reasonably reschedule items without some sort of id
                // and timestamp. I can imagine some random feeds missing one of
                // these things, but any sane podcast feed should have them. If
                // an item doesn't, we just skip it.
                if had_id && had_timestamp && had_enclosure {
                    writer.write_event(Event::Start(start))?;
                    for ev in events {
                        writer.write_event(ev)?;
                    }
                    writer.write_event(Event::End(end))?;
                }
                return Ok(());
            }
            Ok(Event::Start(start)) => {
                let element_tag = start.name();
                let mut start_buf = Vec::new();
                match element_tag {
                    QName(b"guid") | QName(b"id") => {
                        had_id = true;
                        let guid = read_contents(reader, &start)?;

                        if let Some(rescheduled_timestamp) = reschedule.get(&guid) {
                            events.extend(element(start.into_owned(), guid));

                            if let Some((ts_index, ts_start)) = skipped_timestamp.take() {
                                // The original timestamp element was placed before
                                // the guid, so we can write it now that we know the
                                // target timestamp.
                                let timestamp_str = format_timestamp(
                                    ts_start.name().into_inner(),
                                    rescheduled_timestamp,
                                );
                                events.splice(ts_index..ts_index, element(ts_start, timestamp_str));
                            } else {
                                target_timestamp = Some(*rescheduled_timestamp);
                            }
                        } else {
                            reader.read_to_end_into(item_tag, &mut start_buf)?;
                            return Ok(());
                        }
                    }
                    QName(b"pubDate") | QName(b"updated") => {
                        had_timestamp = true;
                        reader.read_to_end_into(element_tag, &mut start_buf)?;
                        if let Some(target_timestamp) = target_timestamp.take() {
                            let timestamp_str =
                                format_timestamp(element_tag.into_inner(), &target_timestamp);
                            events.extend(element(start.to_owned(), timestamp_str));
                        } else {
                            // We haven't seen the guid of this item yet, so we
                            // can't know what the target timestamp is or even
                            // if we want to replay this item.
                            skipped_timestamp = Some((events.len(), start.to_owned()));
                        }
                    }
                    QName(b"enclosure") | QName(b"link") => {
                        if is_audio_enclosure(&start) {
                            had_enclosure = true;
                        }
                        events.push(Event::Start(start.into_owned()));
                    }
                    _ => {
                        events.push(Event::Start(start.into_owned()));
                    }
                };
            }
            Ok(Event::Empty(empty)) => {
                match empty.name() {
                    QName(b"enclosure") | QName(b"link") if is_audio_enclosure(&empty) => {
                        had_enclosure = true;
                    }
                    _ => {}
                };

                events.push(Event::Empty(empty.into_owned()));
            }
            Ok(ev) => {
                events.push(ev.into_owned());
            }
            Err(e) => {
                tracing::error!("Error at position {}: {:?}", reader.buffer_position(), e);
                return Err(e);
            }
        }
        buf.clear();
    }
}

fn format_timestamp(element_tag: &[u8], target_timestamp: &DateTime<Utc>) -> String {
    match element_tag {
        b"pubDate" => target_timestamp.to_rfc2822(),
        _ => target_timestamp.to_rfc3339_opts(SecondsFormat::Secs, true),
    }
}

fn element(start: BytesStart, content: String) -> [Event; 3] {
    let end = start.to_end().into_owned();
    [
        Event::Start(start),
        Event::Text(BytesText::from_escaped(content)),
        Event::End(end),
    ]
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{reschedule::Reschedule, test_helpers::parse_dt};

    use super::rewrite_feed;
    use pretty_assertions::assert_eq;

    fn parse_feed_to_str(
        xml: &str,
        reschedule: &Reschedule<String>,
        title: Option<String>,
    ) -> String {
        let output = rewrite_feed(xml.as_bytes(), reschedule, true, false, &title).unwrap();
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn atom() {
        let xml = include_str!("../tests/data/sample_atom.xml");
        let reschedule = HashMap::from([(
            "urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a".to_string(),
            parse_dt("2021-12-13T16:00:00"),
        )]);
        let output = parse_feed_to_str(xml, &reschedule, Some("Hello World".to_string()));
        let expected = xml
            .replace(
                "        <updated>2003-12-13T18:30:02Z</updated>",
                "        <updated>2021-12-13T16:00:00Z</updated>",
            )
            .replace("<title>Example Feed</title>", "<title>Hello World</title>");
        assert_eq!(output, expected);
    }

    #[test]
    fn rss2() {
        let xml = include_str!("../tests/data/sample_rss_2.0.xml");
        let reschedule = HashMap::from([
            (
                "http://scriptingnews.userland.com/backissues/2002/09/29#When:6:56:02PM"
                    .to_string(),
                parse_dt("2021-12-13T16:00:00"),
            ),
            (
                "http://scriptingnews.userland.com/backissues/2002/09/29#When:12:59:01PM"
                    .to_string(),
                parse_dt("2021-12-20T16:00:00"),
            ),
        ]);
        let output = parse_feed_to_str(xml, &reschedule, None);
        let expected = xml
            .replace(
                "<pubDate>Mon, 30 Sep 2002 01:56:02 GMT</pubDate>",
                "<pubDate>Mon, 13 Dec 2021 16:00:00 +0000</pubDate>",
            )
            .replace(
                "<pubDate>Sun, 29 Sep 2002 19:59:01 GMT</pubDate>",
                "<pubDate>Mon, 20 Dec 2021 16:00:00 +0000</pubDate>",
            )
            .replace(
                "<title>Scripting News</title>",
                "<title>Scripting News (PodReplay)</title>",
            );
        assert_eq!(output, expected);
    }

    #[test]
    fn megaphone() {
        let xml = include_str!("../tests/data/megaphone.xml");
        let reschedule = HashMap::from([
            (
                "612990fc-4f9c-11eb-a6af-e7830eb4fc55".to_string(),
                parse_dt("2022-01-15T16:00:00"),
            ),
            (
                "613b2312-4f9c-11eb-a6af-b700e1b799da".to_string(),
                parse_dt("2022-01-16T16:00:00"),
            ),
            (
                "614f5f12-4f9c-11eb-a6af-cb9557e04485".to_string(),
                parse_dt("2022-01-17T16:00:00"),
            ),
        ]);
        let output = parse_feed_to_str(xml, &reschedule, None);
        let expected = xml
            .replace(
                "<guid isPermaLink=\"false\">\n                <![CDATA[",
                "<guid isPermaLink=\"false\">",
            )
            .replace("]]>\n            </guid>", "</guid>")
            .replace(
                "<itunes:name></itunes:name>",
                "<itunes:name>\n            </itunes:name>",
            )
            .replace(
                "<itunes:category text=\"History\"></itunes:category>",
                "<itunes:category text=\"History\">\n        </itunes:category>",
            )
            .replace(
                "<pubDate>Wed, 15 Dec 2021 08:00:00 -0000</pubDate>",
                "<pubDate>Sat, 15 Jan 2022 16:00:00 +0000</pubDate>",
            )
            .replace(
                "<pubDate>Wed, 22 Dec 2021 08:00:00 -0000</pubDate>",
                "<pubDate>Sun, 16 Jan 2022 16:00:00 +0000</pubDate>",
            )
            .replace(
                "<pubDate>Wed, 29 Dec 2021 08:00:00 -0000</pubDate>",
                "<pubDate>Mon, 17 Jan 2022 16:00:00 +0000</pubDate>",
            )
            .replace(
                "<title>Slow Burn</title>",
                "<title>Slow Burn (PodReplay)</title>",
            );
        assert_eq!(output, expected);
    }

    // We don't explicitely support RSS 0.91 or 0.92 since they don't seem to have
    // an item level pubDate and I doubt they're really used for podcast feeds
    // these days. On the other hand, I'm not making any specific efforts to
    // block them. If they have an <item> element with <updated> or <pubDate>
    // elements, we'll pick them up and things should work ok.
}
