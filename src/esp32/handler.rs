use crate::esp32::{EspMessage, SerialHandler};
use tokio::time::{timeout, Duration};

pub struct EspHandler {
    serial: SerialHandler,
    max_retries: u8,
}

impl EspHandler {
    pub fn new(serial: SerialHandler) -> Self {
        Self { serial, max_retries: 10 }
    }


    pub async fn send_with_retry(&mut self, msg: &EspMessage) -> tokio::io::Result<()> {
    let text = msg.to_string();
    for attempt in 1..=self.max_retries {
        self.serial.send(&text).await?;

        // wait up to 2 seconds for a reply
        match timeout(Duration::from_secs(5), self.serial.read_line()).await {

            // Deal with String
            Ok(Ok(reply)) => {
                if reply.trim() == "ACK" {
                    println!("[OrangePi] Got ACK on attempt {}", attempt);
                    return Ok(());
                }
                if reply.trim() == "ERR" {
                    println!("[OrangePi] Got ERR on attempt {}", attempt);
                    return Ok(());
                }
            }
            Ok(Err(e)) => {
                println!("[OrangePi] Error reading reply: {}", e);
            }

            Err(_) => {
                println!("[OrangePi] Timeout waiting for reply on attempt {}", attempt);
            }
        }

        println!("[OrangePi] Retry {}/{}", attempt, self.max_retries);
    }

    Err(tokio::io::Error::new(tokio::io::ErrorKind::Other, "No ACK received"))
}

    pub async fn receive_message(&mut self) -> tokio::io::Result<EspMessage> {
        let raw = self.serial.read_line().await?;
        Ok(EspMessage::from_string(&raw).unwrap())
    }
}