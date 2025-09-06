use futures::{StreamExt, SinkExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio_tungstenite::tungstenite::protocol::Message;

pub struct Connection {
    pub connected: Arc<AtomicBool>,
}

impl Connection {
    pub fn new() -> Self {
        Connection {
            connected: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Waits until backend establishes a WebSocket connection
    pub async fn wait_for_connection(addr: &str, connected: Arc<AtomicBool>) {
        let listener = TcpListener::bind(addr).await.expect("Failed to bind");
        println!("Waiting for backend WebSocket connection on {}", addr);

        let (stream, _) = listener.accept().await.expect("Failed to accept");
        let ws_stream = accept_async(stream).await.expect("Failed to upgrade to WebSocket");
        println!("Backend connected via WebSocket!");

        connected.store(true, Ordering::SeqCst);

        // Example: send ack to backend
        let (mut write, mut read) = ws_stream.split();
        write.send(Message::Text("{\"status\":\"connected\"}".into()))
            .await
            .unwrap();

        // Just read one message for now (later this will be the listener task)
        if let Some(Ok(msg)) = read.next().await {
            println!("Received first msg from backend: {:?}", msg);
        }
    }
}
