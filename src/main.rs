use axum::{
    Router, body::Bytes, extract::ws::{Message, Utf8Bytes, WebSocket, WebSocketUpgrade}, response::IntoResponse, routing::{any, get}
};
use tokio::sync::{Mutex, broadcast};

use std::{ops::ControlFlow, sync::Arc};
use std::{net::SocketAddr, path::PathBuf};
use tower_http::{
    services::ServeDir,
    trace::{DefaultMakeSpan, TraceLayer},
};

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

//allows to extract the IP of connecting user
use axum::extract::connect_info::ConnectInfo;
use axum::extract::ws::CloseFrame;

//allows to split the websocket stream into separate TX and RX branches
use futures_util::{sink::SinkExt, stream::StreamExt};

use crate::httpclient::poll_lastfm;

mod websocket;
mod httpclient;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
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
    
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    tracing::debug!("Listening on {}", listener.local_addr().unwrap());

    tokio::spawn(poll_lastfm(state.clone()));

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

