mod esp32;
use esp32::{EspHandler, EspMessage, SerialHandler};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {
    // Change this to your ESP32 port
    let port_name = "/dev/pts/6";
    let baud_rate = 115200;

    let serial = SerialHandler::new(port_name, baud_rate)
        .expect("Failed to open serial port");
    let mut esp = EspHandler::new(serial);

    // Example: send a command
    let msg = EspMessage {
        cmd: "MOVE".to_string(),
        motor: Some(1),
        steps: Some(50),
    };
    
    esp.send_message(&msg).await?;

    // Example: read a response message
    let response = esp.receive_message().await?;
    println!("Got message back: {}", response.to_string());

    Ok(())
}