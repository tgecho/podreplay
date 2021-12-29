use axum::{
    body::{Body, BoxBody},
    http::Request,
    Router,
};
use hyper::{Response, StatusCode};
use podreplay::{db::Db, fetch::HttpClient, router::make_router};
use pretty_assertions::assert_eq;
use tower::util::ServiceExt;
use tracing_test::traced_test;

async fn test_app() -> Router {
    let db = Db::new("sqlite::memory:").await.unwrap();
    db.migrate().await.unwrap();

    let http = HttpClient::new();
    make_router(db, http)
}

async fn get(app: Router, uri: &str) -> Response<BoxBody> {
    app.oneshot(Request::builder().uri(uri).body(Body::empty()).unwrap())
        .await
        .unwrap()
}

#[traced_test]
#[tokio::test]
async fn returns_200() {
    let xml = include_str!("../../lib/tests/data/sample_atom.xml");
    let mock = mockito::mock("GET", "/hello").with_body(xml).create();
    let mock_uri = format!("{}/hello", &mockito::server_url());

    let app = test_app().await;

    let uri = format!(
        "/replay?rule=1w&start=2021-10-23T01:09:00Z&now=2021-11-23T01:09:00Z&uri={}",
        mock_uri
    );
    let response = get(app, &uri).await;
    let status = response.status();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();

    let expected = xml.replace(
        "        <updated>2003-12-13T18:30:02Z</updated>",
        "        <updated>2021-10-23T01:09:00Z</updated>",
    );
    assert_eq!(expected, body);
    assert_eq!(status, StatusCode::OK);

    mock.assert();
}

#[traced_test]
#[tokio::test]
async fn returns_304_if_expires_is_in_the_future() {
    let etag = "\"2022-10-23T01:09:00Z|opaquestring\"";
    let app = test_app().await;
    let uri =
        "/replay?rule=1w&start=2021-10-23T01:09:00Z&now=2021-11-23T01:09:00Z&uri=/doesnotmatter";

    let response = app
        .oneshot(
            Request::builder()
                .uri(uri)
                .header("If-None-Match", etag)
                .body(Body::empty())
                .unwrap(),
        )
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
async fn returns_304_if_feed_returns_304() {
    let replay_etag = r#""2021-10-23T01:09:00Z|feed_etag""#;
    let feed_etag = r#""feed_etag""#;

    let mock = mockito::mock("GET", "/returns_304")
        .match_header("If-None-Match", feed_etag)
        .with_header("ETag", feed_etag)
        .with_status(304)
        .create();
    let mock_uri = format!("{}/returns_304", &mockito::server_url());

    let app = test_app().await;

    let uri = format!(
        "/replay?rule=1w&start=2021-10-23T01:09:00Z&now=2021-10-23T01:09:00Z&uri={}",
        mock_uri
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri(uri)
                .header("If-None-Match", replay_etag)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_MODIFIED);
    assert_eq!(response.headers().get("ETag").unwrap(), replay_etag);
    assert_eq!(
        response.headers().get("Expires").unwrap(),
        "Sat, 23 Oct 2021 01:09:00 +0000"
    );

    mock.assert();
}
