use tokio_tungstenite::{connect_async, WebSocketStream, MaybeTlsStream};
use tokio::net::TcpStream;
use url::Url;
use std::error::Error;


/// Connect to a backend (ws:// or wss://) with retries
pub async fn connect_to_backend_with_retry(
    url: &str,
    max_retries: u8,
    delay_secs: u64,
) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, Box<dyn Error>> {
    for attempt in 1..=max_retries {
        println!("Attempting connection ({}/{})...", attempt, max_retries);

        match Url::parse(url) {
            Ok(url) => match connect_async(url.clone()).await {
                Ok((ws_stream, _)) => {
                    println!("✅ Connected to backend at {}", url);
                    return Ok(ws_stream);
                }
                Err(e) => eprintln!("Connection failed: {}. Retrying in {}s...", e, delay_secs),
            },
            Err(e) => eprintln!("Invalid URL {}: {}. Retrying in {}s...", url, e, delay_secs),
        }

        tokio::time::sleep(std::time::Duration::from_secs(delay_secs)).await;
    }

    Err(format!("Could not connect after {} retries", max_retries).into())
}