use axum::{
    extract::{State, ws::{Message, WebSocket, WebSocketUpgrade}}, response::IntoResponse
};
use std::{net::SocketAddr, sync::Arc};

use axum::extract::connect_info::ConnectInfo;

use futures_util::{SinkExt, stream::StreamExt};

use crate::AppState;

pub async fn handle_websocket(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, addr, state))
}

async fn handle_socket(socket: WebSocket, addr: SocketAddr, state: Arc<AppState>) {
    tracing::debug!("New WebSocket connection from {}", addr);

    let (mut sender, mut receiver) = socket.split();

    let mut rx = state.tx.subscribe();

    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg.into())).await.is_err() {
                break;
            }
        }
    });

    // if let Err(e) = sender.send(Message::Text(text)).await {
    //     tracing::error!("Error sending message to {}: {}", addr, e);
    //     break;
    // }

    tracing::debug!("WebSocket connection from {} closed", addr);
}