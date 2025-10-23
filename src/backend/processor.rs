use tokio::time::{self, Duration};
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::tungstenite::Message;
use serde_json::json;
use futures::SinkExt;
use std::sync::Arc;

use tokio::sync::RwLock;
use crate::backend::models::Command;
use crate::backend::session_state::SessionState;

pub struct Processor<S> {
    rx: Receiver<Command>,
    write: S,
    session_state: Arc<RwLock<SessionState>>,
    latest_frame: Arc<RwLock<Vec<u8>>>, 

}

impl<S> Processor<S>
where
     S: SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin + Send + 'static,
{
    pub fn new(rx: Receiver<Command>, write: S, session_state: Arc<RwLock<SessionState>>, latest_frame: Arc<RwLock<Vec<u8>>> ) -> Self {
        Self { rx, write, session_state, latest_frame}
    }

    pub async fn run(&mut self) {
        let mut image_interval = time::interval(Duration::from_millis(66));
        let mut command_interval = time::interval(Duration::from_millis(50));

        loop {
            let connected = {
                let state = self.session_state.read().await;
                if !state.connected && !self.rx.is_empty() {
                    while let Ok(_) = self.rx.try_recv() {
                        // discard messages silently
                    }                    
                    println!("[Processor] Queue cleared... waiting for new session...");
                    continue;
                }
                state.connected
            };

            tokio::select! {
                //  1 Handle queued commands
                _ = command_interval.tick() => {
                    while let Ok(msg) = self.rx.try_recv() {
                        println!("[Processor] Processing command: {:?}", msg);
                        self.handle_command(msg).await;
                    }
                }

                //  2 Send images at 15 fps only if connected
                _ = image_interval.tick(), if connected => {
                    self.send_stream_frame().await;
                }
            }
        }
    }

   async fn handle_command(&mut self, cmd: Command) {
        match cmd {
            Command::Welcome => {
                println!("[Processor] Welcome");
                self.send_ack("Welcome").await;
            }
            Command::Heartbeat => {
                println!("[Processor] Handling Heartbeat");
                self.send_heartbeat().await;
            }
            Command::Move { direction } => {
                println!("[Processor] Moving {}", direction);
                self.send_ack(&format!("Move {}", direction)).await;
                // call your motor control logic here
            }
            Command::Capture => {
                println!("[Processor] Capturing image");
                self.send_image_frame().await;
            }
            Command::SetMicroscope { microscope_id } => {
                println!("[Processor] Set microscope: {}", microscope_id);
                self.send_ack(&format!("SetMicroscope {}", microscope_id)).await;
            }
            Command::Shutdown => {
                println!("[Processor] Shutdown command received");
                self.send_ack("Shutdown").await;
                // optional: trigger shutdown logic
            }
            _ => {
                println!("[Processor] INVALID COMMAND");
            }
        }
    }

    async fn send_heartbeat(&mut self) {
        let heartbeat = json!({
            "type": "Heartbeat",
            "heartbeat": "alive"
        });
        if let Err(e) = self.write.send(Message::Text(heartbeat.to_string())).await {
            eprintln!("[Processor] ❌ Failed to send heartbeat: {}", e);
        } else {
            println!("[Processor] ❤️ Sent heartbeat.");
        }
    }

    async fn send_ack(&mut self, cmd: &str) {
        println!("[Processor] Sending ACK for command: {}", cmd);
        let ack = json!({
            "status": "ACK",
            "command": cmd,
        });
        let _ = self.write.send(Message::Text(ack.to_string())).await;
    }

    async fn send_image_frame(&mut self) {
        println!("[Processor] Sending image frame...");

        let frame_guard = self.latest_frame.read().await;

        if frame_guard.is_empty() {
            println!("[Processor] No frame available to send.");
            return;
        }

        // Encode the binary frame into Base64 for safe JSON transport
        let encoded = base64::encode(&*frame_guard);
        drop(frame_guard); // release lock before sending


        let payload = json!({
            "type": "ImageCaptured",
            "image_data": encoded,
            "format": "jpeg" // or "png", depending on your camera output
        });



        if let Err(e) = self.write.send(Message::Text(payload.to_string())).await {
            eprintln!("[Processor] Failed to send image frame: {}", e);
        } else {
            println!("[Processor] ✅ Sent image frame ({} bytes).", encoded.len());
        }

    }

    async fn send_stream_frame(&mut self) {
        println!("[Processor] Sending stream frame...");

        let frame_guard = self.latest_frame.read().await;

        if frame_guard.is_empty() {
            println!("[Processor] No frame available to send.");
            return;
        }

        // Encode the binary frame into Base64 for safe JSON transport
        let encoded = base64::encode(&*frame_guard);
        drop(frame_guard); // release lock before sending

        let payload = json!({
            "type": "StreamFrame",
            "frame_data": encoded,
            "format": "jpg",
            "timestamp": chrono::Utc::now().timestamp_millis()
        });

        if let Err(e) = self.write.send(Message::Text(payload.to_string())).await {
            eprintln!("[Processor] Failed to send image frame: {}", e);
        } else {
            println!("[Processor] ✅ Sent image frame ({} bytes).", encoded.len());
        }

    }
}
