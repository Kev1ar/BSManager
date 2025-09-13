use tokio::sync::mpsc::Sender;
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;
use futures_util::StreamExt;
use tokio_tungstenite::tungstenite::protocol::Message;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use crate::backend;
/// Spawns a listener that reads JSON messages from the WebSocket and enqueues them.
/// Stops if `shutdown` is set or WebSocket closes/errors.
pub fn spawn_message_listener(
    mut read: futures_util::stream::SplitStream<WebSocketStream<TcpStream>>,
    tx: Sender<backend::models::Command>,
    shutdown: Arc<AtomicBool>,
) {
    tokio::spawn(async move {
        println!("[Listener] Message Listener Online...");
        while !shutdown.load(Ordering::SeqCst) {
            match read.next().await {
                Some(Ok(Message::Text(text))) => {
                    // Try to deserialize JSON into Command
                    match serde_json::from_str::<backend::models::Command>(&text) {
                        Ok(cmd) => {
                            println!("[Listener] Received command: {:?}", cmd);
                            if let Err(e) = tx.send(cmd).await {
                                eprintln!("[Listener] Queue closed: {}", e);
                                break;
                            }
                        }
                        Err(e) => eprintln!("[Listener] Invalid JSON: {}", e),
                    }
                }
                Some(Ok(_)) => {
                    // Ignore non-text messages
                }
                Some(Err(e)) => {
                    eprintln!("[Listener] WebSocket error: {}", e);
                    shutdown.store(true, Ordering::SeqCst);
                    break;
                }
                None => {
                    shutdown.store(true, Ordering::SeqCst);
                    println!("[Listener] WebSocket closed");
                    break;
                }
            }
        }
        println!("[Listener] Exiting message listener task.");
    });
}