use tokio::sync::{mpsc::Receiver, RwLock};
use tokio_tungstenite::tungstenite::protocol::Message;
use futures::SinkExt;
use std::sync::Arc;
use crate::backend::models::Command;
use crate::backend::session_state::SessionState;


/// Spawns a processor task that handles commands from the queue
pub fn spawn_task_processor<W>(
    mut rx: Receiver<Command>,
    mut write: W,
    session_state: Arc<RwLock<SessionState>>,
) 
where
    W: SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin + Send + 'static,
{
    tokio::spawn(async move {
        println!("[Processor] Started.");

        while let Some(message) = rx.recv().await {
            // Only process if session is active
            {
                let state = session_state.read().await;
                if !state.connected {
                    while let Ok(_) = rx.try_recv() {
                        // discard messages silently
                    }                    
                    println!("[Processor] Queue cleared, waiting for new session...");
                    continue;
                }
            }

            println!("[Processor] Processing command: {:?}", message);

            // --- Handle commands ---
            match message.cmd.as_str() {
                // "MOTOR" => {
                //     if let (Some(motor_id), Some(steps)) = (message.motor_id, message.steps) {
                //         println!("[Processor] Moving motor {} from {} to {}", motor_id, steps[0], steps[1]);
                //         tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                //     } else {
                //         eprintln!("[Processor] MOTOR command missing motor_id or steps");
                //     }
                // }
                "Capture" => {
                    println!("[Processor] Capturing image...");
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
                // "AI-RECOG" => {
                //     if message.ai_on.unwrap_or(false) {
                //         println!("[Processor] Running AI recognition...");
                //         tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                //     }
                // }
                // "AI-TRACK" => {
                //     if message.ai_on.unwrap_or(false) {
                //         println!("[Processor] Running AI tracking...");
                //         tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                //     }
                // }
                "DISCONNECT" => {
                    println!("[Processor] Received DISCONNECT. Resetting session...");
                    let mut state = session_state.write().await;
                    state.reset();
                    continue;
                }
                _ => {
                    eprintln!("[Processor] Unknown command: {:?}", message.cmd);
                }
            }

            // --- Send completion ACK ---
            let ack = serde_json::json!({
                "status": "done",
                "cmd": message.cmd,
                // "session_id": message.session_id,
            });

            println!("[Processor] SEND ACK: {}", ack);
            if let Err(e) = write.send(Message::Text(ack.to_string())).await {
                eprintln!("[Processor] Failed to send completion ACK: {}", e);
            }
        }

        println!("[Processor] Exited.");
    });
}
