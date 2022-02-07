use podreplay::{
    config::Config, db::Db, fetch::HttpClient, proxy::ProxyClient, router::make_router,
};
use std::net::{SocketAddr, TcpListener};
use tokio::task::JoinHandle;
use url::Url;

pub struct TestApp {
    pub base_url: Url,
    pub client: reqwest::Client,
    server: JoinHandle<()>,
}

impl TestApp {
    pub async fn new() -> TestApp {
        let config = Config::default();
        let db = Db::new("sqlite::memory:".to_string()).await.unwrap();
        db.migrate().await.unwrap();

        let http = HttpClient::new(config.user_agent.clone(), None);
        let proxy = ProxyClient::new();
        let app = make_router(db, http, proxy, &config);

        let listener = TcpListener::bind("127.0.0.1:0").expect("Could not bind ephemeral socket");
        let addr = listener.local_addr().unwrap();
        let server = tokio::spawn(async move {
            axum::Server::from_tcp(listener)
                .unwrap()
                .serve(app.into_make_service_with_connect_info::<SocketAddr, _>())
                .await
                .unwrap();
        });

        TestApp {
            base_url: Url::parse(&format!("http://{addr}")).unwrap(),
            client: reqwest::Client::new(),
            server,
        }
    }

    pub fn get(&self, path: &str) -> reqwest::RequestBuilder {
        let base_url = Some(&self.base_url);
        let base = Url::options().base_url(base_url);
        let url = base.parse(path).unwrap();
        self.client.get(url)
    }
}

impl Drop for TestApp {
    fn drop(&mut self) {
        tracing::debug!("Dropping test server at {}", self.base_url.as_str());
        self.server.abort()
    }
}
