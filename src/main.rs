mod backend;

use backend::connection::connect_to_backend_with_retry;
use backend::processor::spawn_task_processor;
use backend::listener::spawn_listener;
use backend::session_state::{SessionState};

use futures_util::StreamExt;
use tokio::sync::{mpsc, RwLock};
use tokio::sync::mpsc::Receiver;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("Orange Pi Device Started...");

    // 1️⃣ Connect to backend
    let url = "ws://127.0.0.1:9001";
    let ws_stream = match connect_to_backend_with_retry(url, 5, 2).await {
        Ok(stream) => stream,
        Err(e) => {
            eprintln!("❌ Connection failed: {}", e);
            return;
        }
    };

     // --- Split WebSocket ---
    let (write, read) = ws_stream.split();

    // --- Shared state & queue ---
    let session_state = Arc::new(RwLock::new(SessionState::new()));
    let (tx, rx) = mpsc::channel(100);

    // --- Spawn listener & processor ---
    spawn_listener(read, tx.clone(), Arc::clone(&session_state));
    spawn_task_processor(rx, write, Arc::clone(&session_state));

    // --- Main loop ---
    loop {
        {
            let connected;
            {
                let state = session_state.read().await;
                connected = state.connected;
            }

            if connected {
                println!("User connected. Session active.");

                // Placeholder for camera spawning
                println!("[Main] Spawn camera task here");

                // Wait until disconnected
                loop {
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                    let st = session_state.read().await;
                    if !st.connected {
                        break;
                    }
                }
                // STOP CAMERA TASK HERE
                // RESET SESSION STATE
                // CLEAR QUEUE
                // OTHER SHUTDOWN SIGNALS


                println!("User disconnected. Clear session-specific resources here.");
            }
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

