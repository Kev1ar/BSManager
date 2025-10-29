use tokio::time::{self, Duration};
use tokio::sync::mpsc::Receiver;
use tokio_tungstenite::tungstenite::Message;
use serde_json::json;
use futures::SinkExt;
use std::sync::Arc;

use tokio::sync::RwLock;
use crate::backend::models::Command;
use crate::backend::session_state::SessionState;
use crate::esp32::{EspHandler, EspMessage, SerialHandler};

pub struct Processor<S> {
    rx: Receiver<Command>,
    write: S,
    session_state: Arc<RwLock<SessionState>>,
    latest_frame: Arc<RwLock<Vec<u8>>>, 
    esp: EspHandler, 
}

impl<S> Processor<S>
where
     S: SinkExt<Message, Error = tokio_tungstenite::tungstenite::Error> + Unpin + Send + 'static,
{
    pub fn new(
        rx: Receiver<Command>, 
        write: S, 
        session_state: Arc<RwLock<SessionState>>, 
        latest_frame: Arc<RwLock<Vec<u8>>> ) 
        
        -> Self { 
            let port_name = "/dev/ttyUSB0";
            let baud_rate = 115200;
            let serial = SerialHandler::new(port_name, baud_rate)
                .expect("Failed to open serial port");
            let esp = EspHandler::new(serial);
        Self { rx, write, session_state, latest_frame, esp}
    }

    pub async fn run(&mut self) {

        let mut image_interval = time::interval(Duration::from_millis(17));
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

                //  2 Send images at set FPS
                _ = image_interval.tick(), if connected => {
                    self.send_stream_frame().await;
                }
            }
        }
    }

   async fn handle_command(&mut self, cmd: Command) {
        match cmd {
            Command::Welcome => {
                println!("[Processor] Handling Welcome");
                self.send_ack("Welcome").await;
            }
            Command::Heartbeat => {
                println!("[Processor] Handling Heartbeat");
                self.send_heartbeat().await;
            }
            Command::Move { direction } => {
                println!("[Processor] Moving {}", direction);
               match direction.as_str() {
                    "up" => self.send_esp_command("MOVE", 1, "FWD", 5).await,
                    "down" => self.send_esp_command("MOVE", 1, "BWD", 5).await,
                    "left" => self.send_esp_command("MOVE", 2, "FWD", 5).await,
                    "right" => self.send_esp_command("MOVE", 2, "BWD", 5).await,
                    _ => println!("[Processor] Unknown direction: {}", direction),
                }
                self.send_ack(&format!("Move {}", direction)).await;
            }
            Command::Zoom { direction } => {
                println!("[Processor] Zoom {}", direction);
                // if statements for up or down
                if direction == "in" {
                    self.send_esp_command("MOVE", 3, "FWD", 5).await;
                } else if direction == "out" {
                    self.send_esp_command("MOVE", 3, "BWD", 5).await;
                }
                // send message to associate esp32 motorid
                self.send_ack(&format!("Zoom")).await;
            }
            Command::Capture => {
                println!("[Processor] Capturing image");
                self.send_image_frame().await;
                self.send_ack("Capture").await;
            }
            Command::SetMicroscope { microscope_id } => {
                println!("[Processor] Set microscope: {}", microscope_id);
                {
                    let mut state = self.session_state.write().await;
                    state.microscope_id = Some(microscope_id);
                }
                self.send_ack(&format!("SetMicroscope {}", microscope_id)).await;
            }
            Command::Shutdown => {
                println!("[Processor] Shutdown command received");
                self.send_ack("Shutdown").await;
                
                let mut state = self.session_state.write().await;
                state.cancel_token.cancel();
                state.connected = false;
                
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

    async fn send_esp_command(&mut self, cmd: &str, motor: u8, direction: &str, steps: u32) {
        let msg = EspMessage {
            cmd: cmd.to_string(),
            motor: Some(motor),
            direction: Some(direction.to_string()),
            steps: Some(steps),
        };

        let msg_str = msg.to_string();
        println!("[Processor] Sending ESP command: {}", msg_str);

        if let Err(e) = self.esp.send_with_retry(&msg_str).await {
            eprintln!("[Processor] ❌ Failed to send ESP command '{}': {}", msg_str, e);
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
