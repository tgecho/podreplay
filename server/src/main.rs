mod replay;
mod summary;

use warp::Filter;

#[tokio::main]
async fn main() {
    let summary = warp::path("summary").and(warp::query().map(summary::get));
    let replay = warp::path("replay").and(warp::query().map(replay::get));
    let api = warp::path("api").and(summary);

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["GET", "POST", "DELETE"]);

    let routes = replay.or(api.with(cors));
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
