use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use futures::SinkExt;
use tokio::task;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::stream::SplitSink;
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;

use crate::controllers::camera::Camera;
use crate::controllers::encoder as Encoder;

pub fn spawn_frame_sender(
    mut ws_write: SplitSink<WebSocketStream<TcpStream>, Message>,
    shutdown: Arc<AtomicBool>,
    target_fps: i32,
    cam_index: i32,
    width: i32,
    height: i32,
) {
    task::spawn(async move {
        println!("Frame sender started...");

        let interval = std::time::Duration::from_millis(1000 / target_fps as u64);

        // Init camera
        let mut camera = match Camera::new(cam_index, width, height, target_fps) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to init camera: {:?}", e);
                shutdown.store(true, Ordering::SeqCst);
                return;
            }
        };

        while !shutdown.load(Ordering::SeqCst) {
            println!("Sending Frame...");
            // 1. Capture raw frame
            match camera.capture_frame() {
                Ok(frame) => {
                    // 2. Encode to JPEG (balance speed/quality with ~80-90)
                    match Encoder::encode_to_jpeg(&frame, 85) {
                        Ok(bytes) => {
                            // 3. Send over WebSocket
                            if let Err(e) = ws_write.send(Message::Binary(bytes)).await {
                                shutdown.store(true, Ordering::SeqCst);
                                eprintln!("WebSocket send error: {:?}", e);
                                break;
                            }
                        }
                        Err(e) => eprintln!("Encoding error: {:?}", e),
                    }
                }
                Err(e) => {
                    shutdown.store(true, Ordering::SeqCst);
                    eprintln!("Camera capture error: {:?}", e);
                }
                
            }

            tokio::time::sleep(interval).await;
        }

        println!("Frame sender shutting down...");
    });
}
