use axum::{
    Router, routing::{get}
};
use tokio::sync::{Mutex, broadcast};

use std::{sync::Arc};
use std::{net::SocketAddr};
use tower_http::{trace::{DefaultMakeSpan, TraceLayer}};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::httpclient::poll_lastfm;

mod websocket;
mod httpclient;

#[tokio::main]
async fn main() {
    let port = dotenv::var("PORT").unwrap_or_else(|_| "3000".to_string());
    let lastfm_api_key = dotenv::var("LASTFM_API_KEY").expect("LASTFM_API_KEY must be set in env vars");
    let lastfm_username = dotenv::var("LASTFM_USERNAME").expect("LASTFM_USERNAME must be set in env vars");

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                "info".into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let state = Arc::new(AppState::new());

    let app = Router::new()
        .route("/ws", get(websocket::handle_websocket))
        .with_state(state.clone())
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::default().include_headers(true)),
        );
    
    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port))
        .await
        .unwrap();

    tracing::info!("Listening on {}", listener.local_addr().unwrap());

    tokio::spawn(poll_lastfm(state.clone(), lastfm_api_key, lastfm_username));

    axum::serve(
        listener, 
        app.into_make_service_with_connect_info::<SocketAddr>(),
    ).await.unwrap();
}

#[derive(Clone)]
pub struct AppState {
    pub tx: broadcast::Sender<String>,
    pub lastfm_response: Arc<Mutex<Option<String>>>,
}

impl AppState {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(100);
        Self { tx, lastfm_response: Arc::new(Mutex::new(None)) }
    }
}

