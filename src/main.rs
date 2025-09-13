mod backend;

use tokio::sync::mpsc;
use backend::models::Command;
use backend::{connection::wait_for_connection, listener::spawn_message_listener};
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use futures_util::StreamExt;

#[tokio::main]
async fn main() {
    println!("Orange Pi Image Aquisition BSDMANAGER Started...");
    
    let shutdown_signal = Arc::new(AtomicBool::new(false));

    // Step 1: wait for backend connection
    let ws_stream = wait_for_connection("0.0.0.0:5000").await;

    // Split websocket (for listener & future writer)
    let (write, read) = ws_stream.split();

    // Step 2: setup queue
    let (tx, mut rx) = mpsc::channel::<Command>(100);
                                                                                                                
    // Step 3: start message listener

    spawn_message_listener(read, tx.clone(), shutdown_signal.clone());

    println!("System ready. Waiting for commands...");

    // Step 4 start command processor

    // Step 5: block until shutdown
    while !shutdown_signal.load(Ordering::SeqCst) {
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    }

    println!("Shutdown signal received. Cleaning up...");
}