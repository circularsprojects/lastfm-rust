use std::{sync::Arc};

use crate::AppState;

pub async fn poll_lastfm(state: Arc<AppState>) {
    let client = reqwest::Client::new();
    let mut interval = tokio::time::interval(std::time::Duration::from_secs(5));

    loop {
        interval.tick().await;

        match client.get("https://ws.audioscrobbler.com/2.0/?method=user.getrecenttracks&user=circular_&api_key=&format=json&limit=1").send().await {
            Ok(res) => match res.text().await {
                Ok(body) => {
                    let mut last = state.lastfm_response.lock().await;
                    if last.as_deref() != Some(&body) {
                        tracing::debug!("Last.fm response changed, publishing update");
                        *last = Some(body.clone());
                        let _ = state.tx.send(body);
                    }
                }
                Err(e) => tracing::error!("Failed to read Last.fm response: {}", e),
            },
            Err(e) => tracing::error!("Failed to fetch from Last.fm: {}", e),
        }
    }
}