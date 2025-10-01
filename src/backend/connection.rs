use tokio_tungstenite::{connect_async, tungstenite::protocol::Message, WebSocketStream, MaybeTlsStream};
use tokio::net::TcpStream;
use futures_util::{SinkExt, StreamExt};
use url::Url;
use std::error::Error;
use tokio::sync::mpsc::Sender;

use crate::backend::models::Command;

/// Connect to a backend (ws:// or wss://) with retries
pub async fn connect_to_backend_with_retry(
    url: &str,
    max_retries: u8,
    delay_secs: u64,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn Error>> {
    for attempt in 1..=max_retries {
        println!("Attempting connection ({}/{})...", attempt, max_retries);

        match Url::parse(url) {
            Ok(url) => match connect_async(url.clone()).await {
                Ok((ws_stream, _)) => {
                    println!("✅ Connected to backend at {}", url);
                    return Ok(ws_stream);
                }
                Err(e) => eprintln!("Connection failed: {}. Retrying in {}s...", e, delay_secs),
            },
            Err(e) => eprintln!("Invalid URL {}: {}. Retrying in {}s...", url, e, delay_secs),
        }

        tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
    }

    Err(format!("Could not connect after {} retries", max_retries).into())
}

/// Listener that only consumes messages from the read half and detects ON messages.
/// Other messages can be forwarded to a queue if provided.
pub async fn listen_for_on<S>(
    mut read: futures_util::stream::SplitStream<WebSocketStream<S>>,
    tx: Option<Sender<Command>>,
) -> Result<Command, Box<dyn Error>>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin + Send + 'static,
{
    println!("Waiting for JSON 'ON' message from backend...");

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                println!("Received text: {}", text);

                match serde_json::from_str::<Command>(&text) {
                    Ok(cmd) => {
                        if cmd.cmd.to_uppercase() == "ON" {
                            println!("✅ Received 'ON' command!");
                            return Ok(cmd);
                        } else if let Some(tx) = &tx {
                            // Forward other messages to processor queue if session started
                            if tx.send(cmd).await.is_err() {
                                eprintln!("⚠️ Failed to send to processor queue");
                            }
                        } else {
                            println!("⚠️ Ignoring message (no session yet): {}", text);
                        }
                    }
                    Err(_) => println!("⚠️ Failed to parse JSON: {}", text),
                }
            }
            Ok(Message::Close(_)) => return Err("Backend closed connection".into()),
            Err(e) => return Err(format!("WebSocket error: {:?}", e).into()),
            _ => {}
        }
    }

    Err("WebSocket stream ended before receiving 'ON'".into())
}