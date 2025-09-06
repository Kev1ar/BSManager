mod backend;
use backend::connection::Connection;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let connection = Connection::new();
    let connected = Arc::clone(&connection.connected);

    // Wait until backend connects
    Connection::wait_for_connection("0.0.0.0:5000", connected).await;

    println!("connected...Run listener task now");

    // Just keep program alive
    while connection.connected.load(Ordering::SeqCst) {
        sleep(Duration::from_secs(1)).await;
    }
}
