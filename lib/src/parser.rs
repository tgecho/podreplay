use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;
use std::io::{BufRead, Write};

use crate::replay::Reschedule;

/*
TODO: Consider how/if we can rewrite (or omit?) the pubDate/lastPubDate/ttl/skipHours/skipDays channel elements
 */

struct Writer<W: Write> {
    writer: quick_xml::Writer<W>,
}

impl<W: Write> Writer<W> {
    fn new(writer: quick_xml::Writer<W>) -> Self {
        Writer { writer }
    }

    fn write(&mut self, ev: Event) -> Result<(), quick_xml::Error> {
        match ev {
            Event::CData(data) => {
                // CData contents are being escaped improperly. In order to get
                // a 1:1 output, we need to undo this (and trick quick-xml)
                // before writing.
                // https://github.com/tafia/quick-xml/issues/311
                let unescaped = data.unescaped().unwrap().into_owned();
                let ev = Event::CData(BytesText::from_escaped(&unescaped));
                self.writer.write_event(ev)
            }
            ev => self.writer.write_event(ev),
        }
    }
}

pub fn parse_feed<R: BufRead, W: Write>(
    mut reader: quick_xml::Reader<R>,
    writer: quick_xml::Writer<W>,
    reschedule: &Reschedule,
) {
    let mut writer = Writer::new(writer);
    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(start)) => match start.name() {
                b"item" | b"entry" => {
                    parse_item(start, &mut reader, &mut writer, reschedule).unwrap();
                }
                _ => {
                    writer.write(Event::Start(start)).unwrap();
                }
            },
            Ok(ev) => {
                writer.write(ev).unwrap();
            }
            Err(e) => {
                tracing::error!("Error at position {}: {:?}", reader.buffer_position(), e)
            }
        }
        buf.clear();
    }
}

fn parse_item<B: BufRead, W: Write>(
    start: BytesStart,
    reader: &mut Reader<B>,
    writer: &mut Writer<W>,
    reschedule: &Reschedule,
) -> Result<(), quick_xml::Error> {
    let item_tag = start.name();
    let mut buf = Vec::new();
    let mut events = Vec::new();
    let mut skipped_timestamp: Option<(usize, BytesStart)> = None;
    let mut target_timestamp: Option<String> = None;
    let mut had_id = false;
    let mut had_timestamp = false;
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::End(end)) if end.name() == item_tag => {
                // We can't reasonably reschedule items without some sort of id
                // and timestamp. I can imagine some random feeds missing one of
                // these things, but any sane podcast feed should have them. If
                // an item doesn't, we just skip it.
                if had_id && had_timestamp {
                    writer.write(Event::Start(start))?;
                    for ev in events {
                        writer.write(ev)?;
                    }
                    writer.write(Event::End(end))?;
                }
                return Ok(());
            }
            Ok(Event::Start(start)) => {
                let element_tag = start.name();
                let mut start_buf = Vec::new();
                match element_tag {
                    b"guid" | b"id" => {
                        had_id = true;
                        let guid = reader.read_text(start.name(), &mut start_buf)?;

                        if let Some(rescheduled_timestamp) = reschedule.get(&guid) {
                            // TODO: figure out how to get the correct format here
                            let timestamp_str = rescheduled_timestamp.to_string();
                            events.extend(element(start, guid));

                            if let Some((ts_index, ts_start)) = skipped_timestamp.take() {
                                // The original timestamp element was placed before
                                // the guid, so we can write it now that we know the
                                // target timestamp.
                                events.splice(ts_index..ts_index, element(ts_start, timestamp_str));
                            } else {
                                target_timestamp = Some(timestamp_str);
                            }
                        } else {
                            reader.read_to_end(item_tag, &mut start_buf)?;
                            return Ok(());
                        }
                    }
                    b"pubDate" | b"updated" => {
                        had_timestamp = true;
                        reader.read_to_end(element_tag, &mut start_buf)?;
                        if let Some(target_timestamp) = target_timestamp.take() {
                            events.extend(element(start, target_timestamp.to_string()));
                        } else {
                            // We haven't seen the guid of this item yet, so we
                            // can't know what the target timestamp is or even
                            // if we want to replay this item.
                            skipped_timestamp = Some((events.len(), start.to_owned()));
                        }
                    }
                    _ => {
                        events.push(Event::Start(start.into_owned()));
                    }
                };
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

fn element<'a>(start: BytesStart, content: String) -> [Event<'a>; 3] {
    [
        Event::Start(start.to_owned()),
        Event::Text(BytesText::from_escaped_str(content)),
        Event::End(BytesEnd::owned(start.name().to_owned())),
    ]
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use crate::{replay::Reschedule, test_helpers::parse_dt};

    use super::parse_feed;
    use pretty_assertions::assert_eq;

    fn parse_feed_to_str(xml: &str, reschedule: &Reschedule) -> String {
        let reader = quick_xml::Reader::from_str(xml);
        let mut output = Vec::new();
        let writer = quick_xml::Writer::new_with_indent(&mut output, b' ', 4);
        parse_feed(reader, writer, reschedule);
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn atom() {
        let xml = include_str!("../tests/data/sample_atom.xml");
        let reschedule = HashMap::from([(
            "urn:uuid:1225c695-cfb8-4ebb-aaaa-80da344efa6a".to_string(),
            parse_dt("2021-12-13T16:00:00"),
        )]);
        let output = parse_feed_to_str(xml, &reschedule);
        // TODO: match timestamp formatting
        let expected = xml.replace(
            "        <updated>2003-12-13T18:30:02Z</updated>",
            "        <updated>2021-12-13 16:00:00 UTC</updated>",
        );
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
        let output = parse_feed_to_str(xml, &reschedule);
        // TODO: match timestamp formatting
        let expected = xml
            .replace(
                "<pubDate>Mon, 30 Sep 2002 01:56:02 GMT</pubDate>",
                "<pubDate>2021-12-13 16:00:00 UTC</pubDate>",
            )
            .replace(
                "<pubDate>Sun, 29 Sep 2002 19:59:01 GMT</pubDate>",
                "<pubDate>2021-12-20 16:00:00 UTC</pubDate>",
            );
        assert_eq!(output, expected);
    }

    // We don't explicitely support RSS 0.91 or 0.92 since they don't seem to have
    // an item level pubDate and I doubt they're really used for podcast feeds
    // these days. On the other hand, I'm not making any specific efforts to
    // block them. If they have an <item> element with <updated> or <pubDate>
    // elements, we'll pick them up and things should work ok.
}
