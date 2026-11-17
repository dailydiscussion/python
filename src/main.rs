mod client;
mod handlers;
mod mapping;
mod models;
mod proxy;
mod state;
mod utils;

use axum::{
    routing::{get, post},
    Router,
};
use state::AppState;
use std::net::SocketAddr;
use tower_http::cors::{AllowOrigin, Any, CorsLayer};
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "moviebox_api_rs=info,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    println!("============================================================");
    println!("   🚀 MovieBox Unofficial API Backend (Rust Edition)        ");
    println!("   High Performance • Axum + Tokio • Zero-Copy Proxy       ");
    println!("============================================================");

    let state = AppState::new().await;

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::predicate(|origin, _| {
            let s = origin.to_str().unwrap_or("");
            s.contains("localhost")
                || s.contains("127.0.0.1")
                || s.contains("web.app")
                || s.contains("firebaseapp.com")
        }))
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_credentials(true);

    let proxy_client = state.proxy_client.clone();

    let app = Router::new()
        .route("/", get(handlers::root_handler))
        .route("/health", get(handlers::health_handler))
        .route("/home", get(handlers::home_handler))
        .route("/movies", get(handlers::movies_handler))
        .route("/anime", get(handlers::anime_handler))
        .route("/short-tv", get(handlers::short_tv_handler))
        .route("/kids", get(handlers::kids_handler))
        .route("/education", get(handlers::education_handler))
        .route("/music", get(handlers::music_handler))
        .route("/asian", get(handlers::asian_handler))
        .route("/western", get(handlers::western_handler))
        .route("/nollywood", get(handlers::nollywood_handler))
        .route("/game", get(handlers::game_handler))
        .route("/tab/{id}", get(handlers::tab_handler))
        .route("/search", get(handlers::search_handler))
        .route("/search-suggestions", get(handlers::search_suggestions_handler))
        .route("/detail/{id}", get(handlers::detail_handler))
        .route("/episodes/{id}", get(handlers::episodes_handler))
        .route("/rankings", get(handlers::rankings_handler))
        .route("/stream/{id}", get(handlers::stream_handler))
        .route("/user-info", get(handlers::user_info_handler))
        .route("/login", post(handlers::login_handler))
        .route("/logout", post(handlers::logout_handler))
        .route("/history", get(handlers::history_handler))
        .route("/watchlist", get(handlers::watchlist_handler))
        .route(
            "/proxy-media",
            get(move |headers, query| proxy::proxy_media_handler(headers, query, proxy_client.clone())),
        )
        .layer(cors)
        .with_state(state);

    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(7860);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    info!("MovieBox Rust server listening on http://{}", addr);
    println!(">>> Server listening on http://0.0.0.0:{}", port);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
