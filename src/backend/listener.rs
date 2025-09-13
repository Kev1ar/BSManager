use futures_util::StreamExt;
use tokio::sync::mpsc::Sender;
use tokio_tungstenite::tungstenite::protocol::Message;
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use crate::backend::models::Command;

/// Spawns a message listener that pushes JSON commands into the queue.
/// Also sets the shutdown signal if "off" is received.
pub fn spawn_message_listener(
    mut read: futures_util::stream::SplitStream<WebSocketStream<TcpStream>>,
    tx: Sender<Command>,
    shutdown: Arc<AtomicBool>,
) {
    tokio::spawn(async move {
        while let Some(msg) = read.next().await {
            if let Ok(Message::Text(text)) = msg {
                if text == "off" {
                    println!("[Listener] Received shutdown signal!");
                    shutdown.store(true, Ordering::SeqCst);
                    break;
                }

                match serde_json::from_str::<Command>(&text) {
                    Ok(cmd) => {
                        println!("[Listener] Queued {:?}", cmd);
                        if let Err(e) = tx.send(cmd).await {
                            eprintln!("[Listener] Failed to send to queue: {}", e);
                        }
                    }
                    Err(e) => eprintln!("[Listener] Invalid JSON: {}", e),
                }
            }
        }
    });
}
