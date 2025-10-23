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
                        match message {
                            // Local session control
                            Command::StartStream => {
                                let mut state = session_state.write().await;
                                state.connected = true;
                                println!("[Listener] StartStream received, session active");
                            }
                            Command::StopStream => {
                                let mut state = session_state.write().await;
                                state.reset();
                                println!("[Listener] StopStream received, session reset");
                            }
                            // Forward other commands to processor
                            _ => {
                                if let Err(e) = tx.send(message).await {
                                    eprintln!("[Listener] Processor queue closed: {}", e);
                                    break;
                                }
                            }
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
