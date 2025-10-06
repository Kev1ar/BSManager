use tokio::time::{self, Duration, Instant};
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
}

impl<S> Processor<S>
where
     S: SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin + Send + 'static,
{
    pub fn new(rx: Receiver<Command>, write: S, session_state: Arc<RwLock<SessionState>>,) -> Self {
        Self { rx, write, session_state }
    }

    pub async fn run(&mut self) {
        let mut image_interval = time::interval(Duration::from_millis(500));
        let mut last_frame_time = Instant::now();

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
                // 1️⃣ Handle queued commands
                Some(msg) = self.rx.recv() => {
                    
                    println!("[Processor] Processing command: {}", msg.cmd.as_str());
                    self.handle_command(msg).await;
                }

                // 2️⃣ Send images at 5 fps only if connected
                _ = image_interval.tick(), if connected => {
                    if last_frame_time.elapsed() >= Duration::from_millis(500) {
                        self.send_image_frame().await;
                        last_frame_time = Instant::now();
                    }
                }

                else => {
                    time::sleep(Duration::from_millis(50)).await;
                }
            }
        }
    }

    async fn handle_command(&mut self, msg: Command) {
        println!("⚙️ Executing command: {}", msg.cmd.as_str());
        self.send_ack(&msg.cmd).await;
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
        time::sleep(Duration::from_secs(3)).await;
        // let fake_frame_data = base64::encode("FAKE_IMAGE_DATA");
        // let payload = json!({
        //     "type": "frame",
        //     "data": fake_frame_data,
        //     "timestamp": chrono::Utc::now().timestamp_millis(),
        // });
        // if let Err(e) = self.write.send(Message::Text(payload.to_string())).await {
        //     eprintln!("[Processor] Failed to send image frame: {}", e);
        // }
    }
}
