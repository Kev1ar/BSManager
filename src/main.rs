mod backend;
mod controllers;

use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio::sync::RwLock;

use backend::session_state::SessionState;
use controllers::camera::*;
use std::fs;

#[tokio::main]
async fn main() {
    let session_state = Arc::new(RwLock::new(SessionState::new()));
    let mut cam = Camera::new(640, 480);
    session_state.write().await.connected = true; // Simulate a connected state 
    cam.spawn_task(Arc::clone(&session_state));

    // Save 10 frames, 1 per second
    for i in 1..=10 {
        sleep(Duration::from_secs(1)).await;

        let frame_data = cam.latest_frame();
        let buf = frame_data.lock().unwrap();
        if !buf.is_empty() {
            let filename = format!("frame_{:02}.jpg", i);
            if let Err(e) = fs::write(&filename, &*buf) {
                eprintln!("Failed to save {}: {}", filename, e);
            } else {
                println!("Saved {}", filename);
            }
        } else {
            println!("No frame captured at iteration {}", i);
        }
    }

    // Stop camera
    cam.stop();
    println!("Camera stopped.");
}
