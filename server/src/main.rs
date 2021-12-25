use axum::Server;
use podreplay::db::Db;
use podreplay::fetch::HttpClient;
use podreplay::router::make_router;

#[tokio::main]
async fn main() {
    color_eyre::install().expect("Failed to install color_eyre");

    #[cfg(debug_assertions)]
    if std::env::var_os("RUST_LOG").is_none() {
        std::env::set_var("RUST_LOG", "debug");
    }
    tracing_subscriber::fmt::init();

    let db = Db::new("sqlite://test.sqlite")
        .await
        .expect("Failed to open sqlite");

    let http = HttpClient::new();

    let app = make_router(db, http);

    Server::bind(&"0.0.0.0:3100".parse().expect("Invalid host/port string"))
        .serve(app.into_make_service())
        .await
        .expect("Failed to start server");
}
