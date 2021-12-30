use assert_json_diff::assert_json_eq;
use axum::{
    body::{Body, BoxBody},
    http::Request,
    Router,
};
use hyper::{Response, StatusCode};
use podreplay::{db::Db, fetch::HttpClient, router::make_router};
use pretty_assertions::assert_eq;
use serde_json::{from_slice, json};
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
    let xml = include_str!("../../lib/tests/data/sample_rss_2.0.xml");
    let mock = mockito::mock("GET", "/hello").with_body(xml).create();
    let mock_uri = format!("{}/hello", &mockito::server_url());

    let app = test_app().await;

    let uri = format!("/summary?uri={}", mock_uri);
    let response = get(app, &uri).await;
    let status = response.status();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let actual: serde_json::Value = from_slice(&body).unwrap();
    let expected = json!({
        "uri": mock_uri,
        "title": "Scripting News",
        "items": [
            {
                "id": "http://scriptingnews.userland.com/backissues/2002/09/29#When:12:59:01PM",
                "timestamp": "2002-09-29T19:59:01Z",
                "title": "Joshua Allen: Who loves namespaces?",
            },
            {
                "id": "http://scriptingnews.userland.com/backissues/2002/09/29#When:6:56:02PM",
                "timestamp": "2002-09-30T01:56:02Z",
                "title": "With any luck we should have one or two more days of namespaces stuff here on Scripting Ne...",
            },
        ]
    });
    assert_json_eq!(actual, expected);
    assert_eq!(status, StatusCode::OK);

    mock.assert();
}

#[traced_test]
#[tokio::test]
async fn follows_link_meta_in_html() {
    let mock_html_uri = format!("{}/hello.html", &mockito::server_url());
    let mock_xml_uri = format!("{}/hello.xml", &mockito::server_url());

    let mock_html = mockito::mock("GET", "/hello.html")
        .with_body(format!(
            r#"
                <html>
                    <head>
                        <title>The RSS Blog</title>
                        <link rel="alternate" type="application/rss+xml" title="RSS" href="{}" />
                    </head>
                    <body>
                        <!-- the web page's contents -->
                    </body>
                </html>
            "#,
            mock_xml_uri
        ))
        .create();
    let mock_xml = mockito::mock("GET", "/hello.xml")
        .with_body(include_str!("../../lib/tests/data/sample_rss_2.0.xml"))
        .create();

    let app = test_app().await;

    let uri = format!("/summary?uri={}", mock_html_uri);
    let response = get(app, &uri).await;
    let status = response.status();

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    let actual: serde_json::Value = from_slice(&body).unwrap();
    let expected = json!({
        "uri": mock_xml_uri,
        "title": "Scripting News",
        "items": [
            {
                "id": "http://scriptingnews.userland.com/backissues/2002/09/29#When:12:59:01PM",
                "timestamp": "2002-09-29T19:59:01Z",
                "title": "Joshua Allen: Who loves namespaces?",
            },
            {
                "id": "http://scriptingnews.userland.com/backissues/2002/09/29#When:6:56:02PM",
                "timestamp": "2002-09-30T01:56:02Z",
                "title": "With any luck we should have one or two more days of namespaces stuff here on Scripting Ne...",
            },
        ]
    });
    assert_json_eq!(actual, expected);
    assert_eq!(status, StatusCode::OK);

    mock_html.assert();
    mock_xml.assert();
}
