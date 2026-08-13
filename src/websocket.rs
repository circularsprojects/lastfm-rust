use axum::{
    extract::{State, ws::{Message, WebSocket, WebSocketUpgrade}}, http::HeaderMap, response::IntoResponse
};
use tokio::sync::broadcast;
use std::{net::SocketAddr, sync::Arc};

use axum::extract::connect_info::ConnectInfo;

use futures_util::{SinkExt, stream::StreamExt};

use crate::AppState;

pub async fn handle_websocket(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
    headers: HeaderMap,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state, headers))
}

async fn handle_socket(mut socket: WebSocket, addr: SocketAddr, state: Arc<AppState>, headers: HeaderMap) {
    let proxy_logging = dotenv::var("PROXY_LOGGING").unwrap_or_else(|_| "false".to_string()) == "true";
    if let Some(proxy_header) = headers.get("X-Forwarded-For") && proxy_logging {
        tracing::debug!("New WebSocket connection from {} ({})", proxy_header.to_str().unwrap_or("unknown"), addr);
    } else {
        tracing::debug!("New WebSocket connection from {}", addr);
    }

    let current = state.lastfm_response.lock().await.clone();

    match current {
        None => {
            tracing::error!("No LastFM data available, closing {}", addr);
            socket.close().await.ok();
            return;
        }
        Some(data) => {
            if socket.send(Message::Text(data.into())).await.is_err() {
                tracing::error!("Failed to send initial data to {}", addr);
                return;
            }
        }
    }

    let (mut sender, _receiver) = socket.split();

    let mut rx = state.tx.subscribe();

    loop {
        let msg = rx.recv().await;
        match msg {
            Ok(msg) => {
                if sender.send(Message::Text(msg.into())).await.is_err() {
                    break;
                }
            }
            Err(broadcast::error::RecvError::Lagged(n)) => {
                tracing::warn!("Client {} lagged, missed {} messages", addr, n);
            }
            Err(broadcast::error::RecvError::Closed) => break,
        }
    }

    tracing::debug!("WebSocket connection from {} closed", addr);
}