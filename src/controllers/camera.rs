use opencv::{
    prelude::*,
    videoio,
    core,
    imgcodecs,
};

use tokio::sync::RwLock; 
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tokio_util::sync::CancellationToken;
use crate::backend::session_state::SessionState;

pub struct Camera {
    width: i32,
    height: i32,
    latest_frame: Arc<RwLock<Vec<u8>>>, 
    cancel_token: CancellationToken,
}

impl Camera {
    pub fn new(width: i32, height: i32) -> Self {
        Self {
            width,
            height,
            latest_frame: Arc::new(RwLock::new(Vec::new())),
            cancel_token: CancellationToken::new(),
        }
    }

    /// Spawn the capture task based on session_state
    pub fn spawn_task(&mut self, session_state: Arc<RwLock<SessionState>>) {
        println!("[Camera] Capture task started...");
        let latest_frame = Arc::clone(&self.latest_frame);
        let cancel = self.cancel_token.clone();
        let width = self.width;
        let height = self.height;

        tokio::spawn(async move {

            loop {
                if cancel.is_cancelled() { break; }

                let connected = { session_state.read().await.connected };

                if connected {
                    // Open the camera only when connected
                    let mut capture = match videoio::VideoCapture::new(20, videoio::CAP_V4L2) {
                        Ok(cap) => {println!("Camera opened"); cap},
                        Err(e) => {
                            eprintln!("Failed to open camera: {}", e);
                            sleep(Duration::from_millis(500)).await;
                            continue;
                        }
                    };
                    capture.set(videoio::CAP_PROP_FRAME_WIDTH, width as f64).ok();
                    capture.set(videoio::CAP_PROP_FRAME_HEIGHT, height as f64).ok();

                    // Capture loop while connected
                    while session_state.read().await.connected && !cancel.is_cancelled() {
                        let mut frame = core::Mat::default();
                        if let Ok(read_ok) = capture.read(&mut frame) {
                            if read_ok && !frame.empty() {
                                // Encode asynchronously (optional, see previous optimization)
                                let mut buf = core::Vector::<u8>::new();
                                let params = core::Vector::<i32>::new();
                                if let Ok(_) = imgcodecs::imencode(".jpg", &frame, &mut buf, &params) {
                                    let mut shared = latest_frame.write().await;
                                    *shared = buf.to_vec();
                                }
                            }
                        }
                        sleep(Duration::from_millis(1)).await; // reduce CPU usage
                    }

                    // Drop capture when disconnected
                    drop(capture);
                    println!("[Camera] Camera closed due to disconnect.");
                } else {
                    sleep(Duration::from_millis(50)).await; // less aggressive idle sleep
                }
            }
            println!("[Camera] Capture task stopped.");
        });
    }

    pub fn latest_frame(&self) -> Arc<RwLock<Vec<u8>>> {
        Arc::clone(&self.latest_frame)
    }
}
