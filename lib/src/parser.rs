use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
use quick_xml::Reader;
use std::io::{BufRead, Write};

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
) {
    let mut writer = Writer::new(writer);
    let mut buf = Vec::new();
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::Eof) => break,
            Ok(Event::Start(start)) => match start.name() {
                b"item" | b"entry" => {
                    parse_item(start, &mut reader, &mut writer).unwrap();
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
) -> Result<(), quick_xml::Error> {
    let item_tag = start.name();
    let mut buf = Vec::new();
    let mut events = Vec::new();
    let mut skipped_timestamp: Option<(usize, BytesStart)> = None;
    let mut target_timestamp: Option<&str> = None;
    loop {
        match reader.read_event(&mut buf) {
            Ok(Event::End(end)) if end.name() == item_tag => {
                writer.write(Event::Start(start))?;
                for ev in events {
                    writer.write(ev)?;
                }
                writer.write(Event::End(end))?;
                return Ok(());
            }
            Ok(Event::Start(start)) => {
                let element_tag = start.name();
                let mut start_buf = Vec::new();
                match element_tag {
                    b"guid" | b"id" => {
                        let guid = reader.read_text(start.name(), &mut start_buf)?;
                        // TODO: return without writing if this guid isn't in the wanted set
                        events.extend(element(start, guid));

                        if let Some((ts_index, ts_start)) = skipped_timestamp.take() {
                            // The original timestamp element was placed before
                            // the guid, so we can write it now that we know the
                            // target timestamp.
                            events.splice(ts_index..ts_index, element(ts_start, "wat".to_string()));
                        } else {
                            target_timestamp = Some("wat");
                        }
                    }
                    b"pubDate" | b"updated" => {
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
    use super::parse_feed;
    use pretty_assertions::assert_eq;

    fn parse_feed_to_str(xml: &str) -> String {
        let reader = quick_xml::Reader::from_str(xml);
        let mut output = Vec::new();
        let writer = quick_xml::Writer::new_with_indent(&mut output, b' ', 4);
        parse_feed(reader, writer);
        String::from_utf8(output).unwrap()
    }

    #[test]
    fn atom() {
        let xml = include_str!("../tests/data/sample_atom.xml");
        let output = parse_feed_to_str(xml);
        let expected = xml.replace(
            "        <updated>2003-12-13T18:30:02Z</updated>",
            "        <updated>wat</updated>",
        );
        assert_eq!(output, expected);
    }

    #[test]
    fn rss2() {
        let xml = include_str!("../tests/data/sample_rss_2.0.xml");
        let output = parse_feed_to_str(xml);
        let expected = xml
            .replace(
                "<pubDate>Mon, 30 Sep 2002 01:56:02 GMT</pubDate>",
                "<pubDate>wat</pubDate>",
            )
            .replace(
                "<pubDate>Sun, 29 Sep 2002 19:59:01 GMT</pubDate>",
                "<pubDate>wat</pubDate>",
            );
        assert_eq!(output, expected);
    }

    #[test]
    fn rss92() {
        let xml = include_str!("../tests/data/sample_rss_0.92.xml");
        let output = parse_feed_to_str(xml);
        let expected = xml; // TODO: add pubDate to these?
        assert_eq!(output, expected);
    }

    #[test]
    fn rss91() {
        let xml = include_str!("../tests/data/sample_rss_0.91.xml");
        let output = parse_feed_to_str(xml);
        let expected = xml; // TODO: add pubDate to these?
        assert_eq!(output, expected);
    }
}
