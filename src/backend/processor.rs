use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures::SinkExt;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};

use crate::backend::models::Command;

/// Spawns a task processor that handles commands from the queue
pub fn spawn_task_processor(
    mut rx: Receiver<Command>,
    mut write: impl SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin + Send + 'static,
    shutdown_signal: Arc<AtomicBool>,
) {
    tokio::spawn(async move {
        println!(" Task processor started.");

        while let Some(cmd) = rx.recv().await {
            // Check shutdown
            if shutdown_signal.load(Ordering::SeqCst) {
                println!(" Shutdown signal detected. Exiting processor...");
                break;
            }

            println!("Processing command: {:?}", cmd);

            // --- Handle commands ---
            match cmd.cmd.as_str() {
                "MOTOR" => {
                    if let (Some(motor_id), Some(steps)) = (cmd.motor_id, cmd.steps) {
                        println!("Moving motor {} from {} to {}", motor_id, steps[0], steps[1]);
                        // Simulate motor work
                        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    } else {
                        eprintln!(" MOTOR command missing motor_id or steps");
                    }
                }
                "CAPTURE" => {
                    println!("Capturing image...");
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                }
                "AI-RECOG" => {
                    if cmd.ai_on.unwrap_or(false) {
                        println!("Running AI recognition...");
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    } else {
                        println!("AI recognition command received but ai_on=false");
                    }
                }
                "AI-TRACK" => {
                    if cmd.ai_on.unwrap_or(false) {
                        println!("Running AI tracking...");
                        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                    } else {
                        println!("AI tracking command received but ai_on=false");
                    }
                }
                "DISCONNECT" => {
                    println!("Received DISCONNECT. Triggering shutdown...");
                    shutdown_signal.store(true, Ordering::SeqCst);
                    break;
                }
                _ => {
                    eprintln!(" Unknown command: {:?}", cmd.cmd);
                }
            }

            // --- Send completion ACK ---
            let ack = serde_json::json!({
                "status": "done",
                "cmd": cmd.cmd,
                "session_id": cmd.session_id,
                "meta": cmd.meta
            });

            if let Err(e) = write.send(Message::Text(ack.to_string())).await {
                eprintln!("Failed to send completion ACK: {}", e);
            }
        }

        println!("Task processor exited.");
    });
}
