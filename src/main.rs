mod backend;

use backend::connection::connect_to_backend_with_retry;
use backend::processor::spawn_task_processor;
use backend::listener::spawn_listener;
use backend::session_state::{SessionState};

use futures_util::StreamExt;
use tokio::sync::{mpsc, RwLock};
use std::env;
use dotenv::dotenv;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    println!("Orange Pi Device Started...");

    // 1️⃣ Connect to backend
    dotenv().ok();

    let server_host = env::var("SERVER_HOST").expect("SERVER_HOST not set");
    let server_port = env::var("SERVER_PORT").expect("SERVER_PORT not set");
    let device_name = env::var("DEVICE_NAME").expect("DEVICE_NAME not set");
    let auth_token = env::var("AUTH_TOKEN").expect("AUTH_TOKEN not set");
    let url = format!(
        "wss://{host}:{port}/orangepi/connect?device_name={name}&auth_token={token}",
        host = server_host,
        port = server_port,
        name = device_name,
        token = auth_token
    );

    println!("Connecting to backend at: {}", url);

    // let url = "ws://127.0.0.1:9001";
    let ws_stream = match connect_to_backend_with_retry(&url, 5, 2).await {
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

