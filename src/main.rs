mod esp32;
mod controllers;

use controllers::{camera};
use std::io::{self, Write};
use tokio::io::Result;

use esp32::{EspHandler, EspMessage, SerialHandler};

#[derive(Debug)]
// Mock handler
struct MockEspHandler;

impl MockEspHandler {
    // Simulate sending the string and waiting for a reply
    async fn send_with_retry(&mut self, msg: &str) -> Result<()> {
        println!("(MOCK) Sending message: {}", msg);
        tokio::time::sleep(std::time::Duration::from_millis(2000)).await; // simulate delay
        println!("(MOCK) Reply received!");
        Ok(())
    }
}

#[tokio::main]
async fn main() -> tokio::io::Result<()> {

    println!("Orange Pi ESP32 Mock Started...");

    // let port_name = "/dev/ttyUSB0";
    // let baud_rate = 115200;

    // let serial = SerialHandler::new(port_name, baud_rate)
    //     .expect("Failed to open serial port");
    // let mut esp = EspHandler::new(serial);

    // // Example: send a command
    // let msg = EspMessage {
    //     cmd: "MOVE".to_string(),
    //     direction: "FWD".to_string(),
    //     motor: Some(1),
    //     steps: Some(50),
    // };
    
    // println!("SENDING MESSAGE...");
    // match esp.send_with_retry(&msg).await {

    //     Ok(()) => println!("Command succeeded!"),

    //     Err(e) => {
    //         // INSERT CODE TO HANDLE ERR
    //         eprintln!("Command failed after retries: {}", e);
    //     }
    // };
    let mut esp = MockEspHandler;
    
    loop {
        print!("Enter command (CMD:MOTOR:DIRECTION:STEPS) or 'exit': ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.eq_ignore_ascii_case("exit") { break; }

            match esp.send_with_retry(input).await {
                Ok(()) => println!("Command succeeded!"),
                Err(e) => eprintln!("Command failed: {}", e),
            }
            // Send straight
            // match esp.send_with_retry(&msg).await {

            // Ok(()) => println!("Command succeeded!"),

            // Err(e) => {
            //     // INSERT CODE TO HANDLE ERR
            //     eprintln!("Command failed after retries: {}", e);
            //     }
            // };
    }
    Ok(())
}