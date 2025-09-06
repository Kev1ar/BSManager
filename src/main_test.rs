mod esp32;
mod controllers;

use controllers::{camera};

use esp32::{EspHandler, EspMessage, SerialHandler};

#[tokio::main]
async fn main() -> tokio::io::Result<()> {

    println!("Orange Pi IA Controller Started...");

    // match camera::capture_and_save(20) {
    //     Ok(path) => println!("Image captured successfully: {}", path),
    //     Err(e) => eprintln!("Error capturing image: {:?}", e),
    // }

    // Change this to your ESP32 port
    let port_name = "/dev/ttyS0";
    let baud_rate = 9600;

    let serial = SerialHandler::new(port_name, baud_rate)
        .expect("Failed to open serial port");
    let mut esp = EspHandler::new(serial);

    // Example: send a command
    let msg = EspMessage {
        cmd: "MOVE".to_string(),
        motor: Some(1),
        steps: Some(50),
    };
    
    println!("SENDING MESSAGE...");
    match esp.send_with_retry(&msg).await {

        Ok(()) => println!("Command succeeded!"),

        Err(e) => {
            // INSERT CODE TO HANDLE ERR
            eprintln!("Command failed after retries: {}", e);
        }
    };

    // // Example: read a response message
    // let response = esp.receive_message().await?;
    // println!("Got message back: {}", response.to_string());

    Ok(())
}