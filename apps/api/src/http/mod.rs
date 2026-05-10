mod routes;

use axum::Router;
use tower_http::{
    cors::{AllowOrigin, CorsLayer},
    trace::TraceLayer,
};

use crate::state::AppState;

pub fn router(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(
            state
                .config
                .web_origin
                .parse()
                .expect("validated web origin must parse as header value"),
        ))
        .allow_credentials(true)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    routes::router()
        .with_state(state)
        .layer(cors)
        .layer(TraceLayer::new_for_http())
}
