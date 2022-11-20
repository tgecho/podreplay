mod helpers;

use axum::body::Body;
use helpers::TestApp;
use hyper::{header, StatusCode};
use podreplay::helpers::HeaderMapUtils;
use pretty_assertions::assert_eq;
use tracing_test::traced_test;

#[traced_test]
#[tokio::test]
async fn returns_200_for_atom() {
    let xml = include_str!("../../lib/tests/data/sample_atom.xml");
    let mock = mockito::mock("GET", "/hello").with_body(xml).create();
    let mock_uri = format!("{}/hello", &mockito::server_url());

    let app = TestApp::new().await;

    let path = format!(
        "/replay?rule=1w&start=2021-10-23T01:09:00Z&now=2021-11-23T01:09:00Z&title=My+Custom+Title&uri={mock_uri}"
    );
    let response = app.get(&path).send().await.unwrap();
    let status = response.status();
    let content_type = response.headers().get_string("content-type").unwrap();
    let body = response.bytes().await.unwrap();

    let expected = xml
        .replace(
            "<feed xmlns=\"http://www.w3.org/2005/Atom\">",
            "<feed xmlns=\"http://www.w3.org/2005/Atom\">\n    <itunes:block>Yes</itunes:block>",
        )
        .replace(
            "<title>Example Feed</title>",
            "<title>My Custom Title</title>",
        )
        .replace(
            "        <updated>2003-12-13T18:30:02Z</updated>",
            "        <updated>2021-10-23T01:09:00Z</updated>",
        );
    assert_eq!(expected, body);
    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type, "application/rss+xml");

    mock.assert();
}
#[traced_test]
#[tokio::test]
async fn returns_200_for_rss() {
    let xml = include_str!("../../lib/tests/data/sample_rss_2.0.xml");
    let mock = mockito::mock("GET", "/hello").with_body(xml).create();
    let mock_uri = format!("{}/hello", &mockito::server_url());

    let app = TestApp::new().await;

    let path = format!(
        "/replay?rule=1w&start=2021-10-23T01:09:00Z&now=2021-11-23T01:09:00Z&uri={mock_uri}"
    );
    let response = app.get(&path).send().await.unwrap();
    let status = response.status();
    let content_type = response.headers().get_string("content-type").unwrap();
    let body = response.bytes().await.unwrap();

    let expected = xml
        .replace(
            "\n\t\t\t<pubDate>Mon, 30 Sep 2002 01:56:02 GMT</pubDate>",
            "\n\t\t\t<pubDate>Sat, 30 Oct 2021 01:09:00 +0000</pubDate>",
        )
        .replace(
            "\n\t\t\t<pubDate>Sun, 29 Sep 2002 19:59:01 GMT</pubDate>",
            "\n\t\t\t<pubDate>Sat, 23 Oct 2021 01:09:00 +0000</pubDate>",
        )
        .replace(
            "<channel>",
            "<channel>\n        <itunes:block>Yes</itunes:block>",
        )
        .replace(
            "<title>Scripting News</title>",
            "<title>Scripting News (PodReplay)</title>",
        );
    assert_eq!(expected, body);
    assert_eq!(status, StatusCode::OK);
    assert_eq!(content_type, "application/rss+xml");

    mock.assert();
}

#[traced_test]
#[tokio::test]
async fn returns_304_if_expires_is_in_the_future() {
    let etag = "\"2022-10-23T01:09:00Z|opaquestring\"";

    let app = TestApp::new().await;

    let path =
        "/replay?rule=1w&start=2021-10-23T01:09:00Z&now=2021-11-23T01:09:00Z&uri=/doesnotmatter";
    let response = app
        .get(path)
        .header("If-None-Match", etag)
        .body(Body::empty())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    assert_eq!(response.headers().get("ETag").unwrap(), etag);
    assert_eq!(
        response.headers().get("Expires").unwrap(),
        "Sun, 23 Oct 2022 01:09:00 +0000"
    );
}

#[traced_test]
#[tokio::test]
async fn returns_400_if_now_is_too_far_in_the_future() {
    let app = TestApp::new().await;
    let path =
        "/replay?rule=1w&start=2021-10-23T01:09:00Z&now=2024-11-23T01:09:00Z&uri=/doesnotmatter";
    let response = app.get(path).body(Body::empty()).send().await.unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[traced_test]
#[tokio::test]
async fn returns_304_if_feed_returns_304() {
    let replay_etag = r#""2021-10-23T01:09:00Z|feed_etag""#;
    let feed_etag = r#""feed_etag""#;

    let mock = mockito::mock("GET", "/returns_304")
        .match_header("If-None-Match", feed_etag)
        .with_header("ETag", feed_etag)
        .with_status(304)
        .create();
    let mock_uri = format!("{}/returns_304", &mockito::server_url());

    let app = TestApp::new().await;

    let path = format!(
        "/replay?rule=1w&start=2021-10-23T01:09:00Z&now=2021-10-23T01:09:00Z&uri={mock_uri}"
    );

    let response = app
        .get(&path)
        .header("If-None-Match", replay_etag)
        .body(Body::empty())
        .send()
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    assert_eq!(response.headers().get(header::ETAG).unwrap(), replay_etag);
    assert_eq!(
        response.headers().get(header::EXPIRES).unwrap(),
        "Sat, 23 Oct 2021 01:09:00 +0000"
    );

    mock.assert();
}
