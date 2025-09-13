use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::WebSocketStream;
use tokio::net::TcpStream;

pub async fn wait_for_connection(addr: &str) -> WebSocketStream<TcpStream> {
    let listener = TcpListener::bind(addr).await.expect("Failed to bind");
    println!("Waiting for backend WebSocket connection on {}", addr);

    let (stream, _) = listener.accept().await.expect("Failed to accept");

    let ws_stream = accept_async(stream).await.expect("Failed to upgrade WebSocket");
    println!("✅ Backend connected!");

    ws_stream
}

