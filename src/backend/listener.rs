use tokio::sync::{mpsc, RwLock};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures::StreamExt;
use std::sync::Arc;
use crate::backend::models::Command;
use crate::backend::session_state::SessionState;

pub async fn run_listener<R>(
    mut read: R,
    tx: mpsc::Sender<Command>,
    session_state: Arc<RwLock<SessionState>>,
) 
where
    R: futures::Stream<Item = Result<Message, tokio_tungstenite::tungstenite::Error>> + Unpin + Send + 'static,
{
    println!("[Listener] Online");

    while let Some(msg) = read.next().await {
        match msg {
            Ok(Message::Text(text)) => {
                match serde_json::from_str::<Command>(&text) {
                    Ok(message) => {
                        // Handle ON/OFF commands
                        if message.cmd.to_uppercase() == "ON" {
                            let mut state = session_state.write().await;
                            state.connected = true;
                            println!("[Listener] ON received, session active");
                            continue;
                        } else if message.cmd.to_uppercase() == "OFF" {
                            let mut state = session_state.write().await;
                            state.reset();
                            println!("[Listener] OFF received, session reset");
                            continue;
                        }

                        // Forward to processor queue w/ error handling
                        if let Err(_) = tx.send(message).await {
                            eprintln!("[Listener] Processor queue closed");
                        }
                    }
                    Err(e) => {
                        eprintln!("[Listener] Failed to parse JSON: {}", e);
                    }
                }
            }
            Ok(_) => {}
            Err(e) => {
                eprintln!("[Listener] WebSocket error: {}", e);
                break;
            }
        }
    }

    println!("[Listener] Closed");
}
