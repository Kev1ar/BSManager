mod backend;
mod controllers;

use backend::connection::connect_to_backend_with_retry;
use controllers::camera::Camera;
use backend::processor_v2::Processor;
use backend::listener::run_listener;
use backend::session_state::{SessionState};

use futures_util::StreamExt;
use tokio::sync::{mpsc, RwLock};
use std::env;
use dotenv::dotenv;
use std::sync::Arc;

use minifb::{Key, Window, WindowOptions};
use std::thread;
use std::time::Duration;


#[tokio::main]
async fn main() {
    println!("Orange Pi Device Started...");

    // 1️⃣ Connect to backend
    // dotenv().ok();

    // let server_host = env::var("SERVER_HOST").expect("SERVER_HOST not set");
    // let server_port = env::var("SERVER_PORT").expect("SERVER_PORT not set");
    // let device_name = env::var("DEVICE_NAME").expect("DEVICE_NAME not set");
    // let auth_token = env::var("AUTH_TOKEN").expect("AUTH_TOKEN not set");
    // let url = format!(
    //     "wss://{host}:{port}/orangepi/connect?device_name={name}&auth_token={token}",
    //     host = server_host,
    //     port = server_port,
    //     name = device_name,
    //     token = auth_token
    // );

    // println!("Connecting to backend at: {}", url);

    let url = "ws://127.0.0.1:9001"; // Local testing
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

    let mut camera = Camera::new(640, 480); // width x height
    camera.spawn_task(Arc::clone(&session_state));

    // --- Spawn listener & processor ---
    let listener_state = Arc::clone(&session_state);
    tokio::spawn(async move {
        run_listener(read, tx, listener_state).await;
    });

    let processor_state = Arc::clone(&session_state);
    tokio::spawn(async move {
        let mut processor = Processor::new(rx, write, processor_state);
        processor.run().await;
    });
    
 
    // --- Main loop ---
    loop {
        let connected = {
            let state = session_state.read().await;
            state.connected
        }; 
        if connected {
            println!("User connected. Session active.");

            // Placeholder for camera spawning


            // Wait until disconnected
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                let st = session_state.read().await;
                if !st.connected {
                    break;
                }
            }

            println!("User disconnected. Clear session-specific resources here.");
        }
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    }
}

