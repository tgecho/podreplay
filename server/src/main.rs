use std::net::SocketAddr;

use axum::Server;
use podreplay::config::Config;
use podreplay::db::Db;
use podreplay::fetch::HttpClient;
use podreplay::router::make_router;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");

    let config = Config::new().unwrap_or_else(|errors| {
        for err in errors {
            eprintln!("Config error: {}", err);
        }
        panic!("Invalid config");
    });

    #[cfg(debug_assertions)]
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", &config.log_level);
    }
    tracing_subscriber::fmt::init();

    let db = Db::new(config.database_url.clone())
        .await
        .unwrap_or_else(|err| panic!("Failed to open {} ({})", config.database_url, err));

    db.migrate().await.expect("Failed to run migrations");

    let http = HttpClient::new(config.user_agent.clone());

    let app = make_router(db, http, &config);
    let addr = SocketAddr::new(config.host, config.port);
    Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap_or_else(|err| panic!("Failed to start server {} ({})", addr, err));
}
