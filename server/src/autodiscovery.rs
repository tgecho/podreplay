use async_recursion::async_recursion;
use hyper::body::Buf;
use itertools::Itertools;
use kuchiki::{parse_html, traits::TendrilSink};
use lazy_static::lazy_static;
use percent_encoding::percent_decode_str;
use podreplay_lib::FeedSummary;
use regex::Regex;
use serde::Deserialize;
use std::{
    fmt::Display,
    hash::Hash,
    io::{BufRead, Cursor, Seek},
    str::from_utf8,
};
use thiserror::Error;
use url::Url;

use crate::{
    fetch::{FetchException, Fetched, HttpClient},
    helpers::MyIterUtils,
};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub enum FeedUrl {
    Unknown(Url),
    GoogleLink(Url),
    ApplePodcastId(String),
}

impl Display for FeedUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FeedUrl::Unknown(url) | FeedUrl::GoogleLink(url) => {
                write!(f, "{}", url.as_str())
            }
            FeedUrl::ApplePodcastId(id) => write!(f, "ApplePodcastId:{}", id),
        }
    }
}

#[derive(Deserialize, Debug)]
struct ApplePodcastEntity {
    #[serde(rename = "feedUrl")]
    feed_url: String,
}
#[derive(Deserialize, Debug)]
struct ApplePodcastEntityResults {
    results: Vec<ApplePodcastEntity>,
}

pub struct Autodiscovered {
    pub summary: FeedSummary,
    pub etag: Option<String>,
}

#[derive(Error, Debug)]
pub enum AutodiscoveryException {
    #[error("Unexpected internal error")]
    Fetch(#[from] FetchException),
    #[error("Unexpected internal error")]
    Io(#[from] std::io::Error),
    #[error("Autodiscovery failed")]
    Failed,
}

impl FeedUrl {
    pub fn new(url: Url) -> Self {
        // just in case there's a known URL type embedded in some sort of
        // referral/redirect style link
        if let Ok(decoded) = percent_decode_str(url.as_str()).decode_utf8() {
            if let Some(url) = get_google_podcast_feed_url(&decoded) {
                return FeedUrl::GoogleLink(url);
            } else if let Some(id) = get_apple_podcast_id(&decoded) {
                return FeedUrl::ApplePodcastId(id);
            }
        }
        FeedUrl::Unknown(url)
    }

    #[async_recursion]
    async fn get(
        &self,
        client: &HttpClient,
        etag: Option<String>,
    ) -> Result<Fetched, FetchException> {
        match self {
            FeedUrl::Unknown(url) | FeedUrl::GoogleLink(url) => {
                client.get(url.as_str(), etag).await
            }
            FeedUrl::ApplePodcastId(id) => {
                let api_url =
                    format!("https://itunes.apple.com/lookup?media=podcast&entity=podcast&id={id}");
                let api_resp = client.get(&api_url, None).await?;
                let response: ApplePodcastEntityResults =
                    serde_json::from_reader(api_resp.body.reader())?;
                let feed_url = FeedUrl::new(Url::parse(
                    &response
                        .results
                        .get(0)
                        .ok_or(FetchException::Unknown)?
                        .feed_url,
                )?);
                feed_url.get(client, None).await
            }
        }
    }

    pub async fn attempt_autodiscovery(
        &self,
        client: &HttpClient,
        etag: Option<String>,
    ) -> Result<Autodiscovered, AutodiscoveryException> {
        // if the initial request fails, there isn't much we can do
        let first = self.get(client, etag).await?;

        let mut reader = Cursor::new(first.body);
        // if we're able to parse a valid feed summary, return it
        if let Ok(summary) = FeedSummary::new(first.url.to_string(), &mut reader) {
            return Ok(Autodiscovered {
                summary,
                etag: first.etag,
            });
        }
        // otherwise rewind the reader so we can search it as html
        reader.rewind()?;

        let urls = find_feed_links(&mut reader, first.url.as_str())
            .pipe(prioritize_and_dedup_feed_urls)
            .take(5);

        let mut top: Option<FeedSummary> = None;
        for url in urls {
            tracing::debug!("Attempting to autodiscover from {}", url);
            if let Some(candidate) = get_summary(client, url).await {
                match &top {
                    Some(t) if candidate.len() > t.len() => {
                        top.replace(candidate);
                    }
                    None => {
                        top.replace(candidate);
                    }
                    _ => {}
                };
            }
        }

        top.map_or(Err(AutodiscoveryException::Failed), |summary| {
            Ok(Autodiscovered {
                summary,
                etag: None,
            })
        })
    }
}

async fn get_summary(client: &HttpClient, url: FeedUrl) -> Option<FeedSummary> {
    url.get(client, None).await.ok().and_then(|fetched| {
        FeedSummary::new(fetched.url.to_string(), &mut fetched.body.reader()).ok()
    })
}

fn find_feed_links<R: BufRead>(reader: &mut R, origin: &str) -> impl Iterator<Item = FeedUrl> {
    parse_html()
        .from_utf8()
        .read_from(reader)
        .ok()
        .map(|doc| {
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

            let mut links = Vec::new();

            if let Ok(a_links) = doc.select("a, link") {
                for link in a_links {
                    let attrs = link.attributes.borrow();
                    if let Some(url) = attrs
                        .get("href")
                        .and_then(|href| base.parse(href).ok())
                        .map(FeedUrl::new)
                    {
                        match url {
                            FeedUrl::Unknown(url) => {
                                lazy_static! {
                                    static ref MAYBE_FEED_RE: Regex =
                                        Regex::new(r#"(?i)\b(feed|subscribe|rss|atom)\b"#).unwrap();
                                }
                                if MAYBE_FEED_RE.is_match(&link.as_node().to_string()) {
                                    links.push(FeedUrl::Unknown(url));
                                }
                            }
                            known => {
                                links.push(known);
                            }
                        }
                    }
                }
            }

            links
        })
        .unwrap_or_default()
        .into_iter()
}

fn prioritize_and_dedup_feed_urls<I: Iterator<Item = FeedUrl>>(
    urls: I,
) -> impl Iterator<Item = FeedUrl> {
    urls.sorted_by_key(|url| match url {
        // Google podcast links can be decoded without http requests and they
        // pretty unambigously contain the podcast feed URL
        FeedUrl::GoogleLink(_) => 0,
        // This requires a lookup of the ID to get the final feed URL, but it's
        // a pretty reliable method.
        FeedUrl::ApplePodcastId(_) => 1,
        // ¯\_(ツ)_/¯
        FeedUrl::Unknown(_) => 2,
    })
    // we may have found the same feed URL and/or Apple podcast ID multiple
    // times, so we dedup by the captured string.
    .unique_by(|url| match url {
        FeedUrl::GoogleLink(url) | FeedUrl::Unknown(url) => url.to_string(),
        FeedUrl::ApplePodcastId(id) => id.to_string(),
    })
}

lazy_static! {
    static ref GOOGLE_URL_RE: Regex = Regex::new(r#"\.google\.com/feed/(?P<feed>\w+)\b"#).unwrap();
    static ref ITUNES_URL_RE: Regex =
        Regex::new(r#"\.apple\.com/(\w+/)?podcast/[^/]+/id(?P<id>\w+)\b"#).unwrap();
}

fn get_google_podcast_feed_url(url: &str) -> Option<Url> {
    let base64_feed_url = GOOGLE_URL_RE.captures(url)?.name("feed")?.as_str();
    let feed_url_bytes = base64::decode(base64_feed_url).ok()?;
    let feed_url = from_utf8(&feed_url_bytes).ok()?;
    feed_url.parse().ok()
}

fn get_apple_podcast_id(url: &str) -> Option<String> {
    let matched = ITUNES_URL_RE.captures(url)?.name("id");
    Some(matched?.as_str().to_string())
}

#[cfg(test)]
mod test {

    use crate::{autodiscovery::FeedUrl, helpers::MyIterUtils};
    use pretty_assertions::assert_eq;
    use url::Url;

    use super::{find_feed_links, prioritize_and_dedup_feed_urls};

    #[test]
    fn test_google_url_extraction() {
        assert_eq!(
            FeedUrl::new(Url::parse("https://podcasts.google.com/feed/aHR0cHM6Ly9mZWVkcy50aGlzaXNjcmltaW5hbC5jb20vQ3JpbWluYWxTaG93").unwrap()),
            FeedUrl::GoogleLink(Url::parse("https://feeds.thisiscriminal.com/CriminalShow").unwrap())
        )
    }

    #[test]
    fn extract_apple_podcast_id_from_old_style() {
        assert_eq!(
            FeedUrl::new(
                Url::parse("https://geo.itunes.apple.com/us/podcast/serial/id917918570?mt=2")
                    .unwrap()
            ),
            FeedUrl::ApplePodcastId("917918570".to_string())
        );
    }

    #[test]
    fn extract_apple_podcast_id_from_new_style() {
        assert_eq!(
            FeedUrl::new(
                Url::parse("https://podcasts.apple.com/us/podcast/serial/id917918570?uo=4&mt=2")
                    .unwrap()
            ),
            FeedUrl::ApplePodcastId("917918570".to_string())
        );
    }

    #[test]
    fn extract_apple_podcast_id_from_new_style_2() {
        assert_eq!(
            FeedUrl::new(
                Url::parse("https://podcasts.apple.com/podcast/the-memory-palace/id299436963")
                    .unwrap()
            ),
            FeedUrl::ApplePodcastId("299436963".to_string())
        );
    }

    #[test]
    fn extract_apple_podcast_id_from_obfuscated_redirect_url() {
        assert_eq!(
            FeedUrl::new(
                Url::parse("https://go.redirectingat.com/?id=74968X1525078&xs=1&url=https%3A%2F%2Fpodcasts.apple.com%2Fus%2Fpodcast%2Famerican-radical%2Fid1596796171").unwrap()),
            FeedUrl::ApplePodcastId("1596796171".to_string())
        );
    }

    #[test]
    fn find_links_defaults_to_empty() {
        let html = r#""#;
        let found: Vec<_> = find_feed_links(&mut html.as_bytes(), "http://example.com").collect();
        assert_eq!(found, vec![]);
    }

    #[test]
    fn find_links() {
        let html = r#"
            <link rel="alternate" type="application/rss+xml" href="relative.xml" />
            <link rel="alternate" type="application/rss+xml" href="http://example.org/absolute.xml" />
            <a href="https://podcasts.apple.com/us/podcast/serial/id917918570?uo=4&mt=2">Apple Podcasts</a>
            <a href="https://podcasts.google.com/feed/aHR0cHM6Ly9mZWVkcy50aGlzaXNjcmltaW5hbC5jb20vQ3JpbWluYWxTaG93">Google Podcasts</a>
            <a href="relative.xml">Subscribe</a>
            <a href="http://example.org/absolute.xml">Feed</a>
            <a href="feed.xml">Rss</a>
        "#;
        let found: Vec<_> = find_feed_links(&mut html.as_bytes(), "http://example.com").collect();
        assert_eq!(
            found,
            vec![
                FeedUrl::Unknown(Url::parse("http://example.com/relative.xml").unwrap()),
                FeedUrl::Unknown(Url::parse("http://example.org/absolute.xml").unwrap()),
                FeedUrl::ApplePodcastId("917918570".to_string()),
                FeedUrl::GoogleLink(
                    Url::parse("https://feeds.thisiscriminal.com/CriminalShow").unwrap()
                ),
                FeedUrl::Unknown(Url::parse("http://example.com/relative.xml").unwrap()),
                FeedUrl::Unknown(Url::parse("http://example.org/absolute.xml").unwrap()),
                FeedUrl::Unknown(Url::parse("http://example.com/feed.xml").unwrap()),
            ]
        );
    }

    #[test]
    fn find_links_with_base() {
        let html = r#"
            <base href="http://example.org/">
            <link rel="alternate" type="application/rss+xml" href="relative.xml" />
            <link rel="alternate" type="application/rss+xml" href="http://example.com/absolute.xml" />
            <a href="https://podcasts.apple.com/us/podcast/serial/id917918570?uo=4&mt=2">Apple Podcasts</a>
            <a href="https://podcasts.google.com/feed/aHR0cHM6Ly9mZWVkcy50aGlzaXNjcmltaW5hbC5jb20vQ3JpbWluYWxTaG93">Google Podcasts</a>
            <a href="relative.xml">Subscribe</a>
            <a href="http://example.com/absolute.xml">Feed</a>
            <a href="feed.xml">Rss</a>
        "#;
        let found: Vec<_> = find_feed_links(&mut html.as_bytes(), "http://example.com").collect();
        dbg!(found.len());
        assert_eq!(
            found,
            vec![
                FeedUrl::Unknown(Url::parse("http://example.org/relative.xml").unwrap()),
                FeedUrl::Unknown(Url::parse("http://example.com/absolute.xml").unwrap()),
                FeedUrl::ApplePodcastId("917918570".to_string()),
                FeedUrl::GoogleLink(
                    Url::parse("https://feeds.thisiscriminal.com/CriminalShow").unwrap()
                ),
                FeedUrl::Unknown(Url::parse("http://example.org/relative.xml").unwrap()),
                FeedUrl::Unknown(Url::parse("http://example.com/absolute.xml").unwrap()),
                FeedUrl::Unknown(Url::parse("http://example.org/feed.xml").unwrap()),
            ]
        );
    }

    #[test]
    fn dedup_links() {
        let links: Vec<_> = [
            FeedUrl::Unknown(Url::parse("http://example.com/relative.xml").unwrap()),
            FeedUrl::Unknown(Url::parse("http://example.com/absolute.xml").unwrap()),
            FeedUrl::ApplePodcastId("917918570".to_string()),
            FeedUrl::GoogleLink(
                Url::parse("https://feeds.thisiscriminal.com/CriminalShow").unwrap(),
            ),
            FeedUrl::Unknown(Url::parse("http://example.com/relative.xml").unwrap()),
            FeedUrl::Unknown(Url::parse("http://example.com/absolute.xml").unwrap()),
            FeedUrl::Unknown(Url::parse("http://example.com/feed.xml").unwrap()),
        ]
        .into_iter()
        .pipe(prioritize_and_dedup_feed_urls)
        .collect();

        assert_eq!(
            links,
            vec![
                FeedUrl::GoogleLink(
                    Url::parse("https://feeds.thisiscriminal.com/CriminalShow").unwrap()
                ),
                FeedUrl::ApplePodcastId("917918570".to_string()),
                FeedUrl::Unknown(Url::parse("http://example.com/relative.xml").unwrap()),
                FeedUrl::Unknown(Url::parse("http://example.com/absolute.xml").unwrap()),
                FeedUrl::Unknown(Url::parse("http://example.com/feed.xml").unwrap()),
            ]
        );
    }
}
