use cnm::{
    config::Config,
    errors::handle_rejection,
    router::{band::band_routes, org::org_routes, user::user_routes},
};
use warp::Filter;

#[tokio::main]
async fn main() {
    let config = Config::retrieve().expect("Unable to retrieve configuration file");
    let band_routes = warp::path("band").and(band_routes(config.clone()));
    let org_routes = warp::path("org").and(org_routes(config.clone()));
    let user_routes = warp::path("user").and(user_routes(config));
    let cors = warp::cors().allow_any_origin();
    let api = warp::path("api")
        .and(band_routes.or(org_routes).or(user_routes))
        .with(cors)
        .recover(handle_rejection);
    warp::serve(api).run(([127, 0, 0, 1], 3030)).await;
}
