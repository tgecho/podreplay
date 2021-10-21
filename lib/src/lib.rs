use chrono::{DateTime, Utc};
use feed_rs::{model::Feed as ParsedFeed, parser};
use serde::Serialize;

pub fn parse_feed(source: &[u8], uri: Option<&str>) -> Feed {
    let feed = parser::parse_with_uri(source, uri).unwrap();
    Feed::new(feed, uri.map(|uri| uri.to_string()))
}

pub struct Feed {
    feed: ParsedFeed,
    uri: Option<String>,
}

impl Feed {
    fn new(feed: ParsedFeed, uri: Option<String>) -> Self {
        Feed { feed, uri }
    }

    pub fn into_summary(self) -> FeedSummary {
        self.into()
    }
}

#[derive(Serialize)]
pub struct FeedSummary {
    title: Option<String>,
    uri: Option<String>,
    items: Vec<FeedSummaryItem>,
}

#[derive(Serialize)]
pub struct FeedSummaryItem {
    title: Option<String>,
    timestamp: i64,
}

impl From<Feed> for FeedSummary {
    fn from(feed: Feed) -> Self {
        let title = feed.feed.title.map(|t| t.content);
        let mut items = Vec::with_capacity(feed.feed.entries.len());
        for entry in feed.feed.entries {
            if let Some(timestamp) = entry.published.or(entry.updated) {
                items.push(FeedSummaryItem {
                    title: entry.title.map(|t| t.content),
                    timestamp: timestamp.timestamp(),
                });
            }
        }
        items.reverse();
        items.sort_by_key(|i| i.timestamp);
        FeedSummary {
            title,
            uri: feed.uri,
            items,
        }
    }
}

#[cfg(test)]
mod tests {

    use feed_rs::parser;
    use rss::{ChannelBuilder, ItemBuilder};

    #[test]
    fn it_works() {
        let bytes: &'static [u8] = include_bytes!("../../server/src/serial.xml");
        let parsed =
            parser::parse_with_uri(bytes, Some("https://feeds.simplecast.com/xl36XBC2")).unwrap();

        dbg!(&parsed.entries.last());

        let mut items = Vec::with_capacity(parsed.entries.len());
        for entry in parsed.entries {
            items.push(
                ItemBuilder::default()
                    .title(entry.title.unwrap().content)
                    .build()
                    .unwrap(),
            );
        }
        let channel = ChannelBuilder::default()
            .title(parsed.title.unwrap().content)
            // .link(parsed.links)
            .description(parsed.description.unwrap().content)
            .items(items)
            .build()
            .unwrap();
        let buf = Vec::new();
        let buf = channel.write_to(buf).unwrap();
        let string = String::from_utf8(buf).unwrap();
        dbg!(string);
        assert_eq!(1, 4);
    }
}
