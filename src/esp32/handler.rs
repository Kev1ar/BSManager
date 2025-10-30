use crate::esp32::{EspMessage, SerialHandler};
use tokio::time::{timeout, Duration};
use tokio::io;

pub struct EspHandler {
    serial: SerialHandler,
    max_retries: u8,
    retry_delay: Duration,
    ack_timeout: Duration,
}

impl EspHandler {
    pub fn new(serial: SerialHandler) -> Self {
        Self { 
            serial,
            max_retries: 3,
            retry_delay: Duration::from_millis(30), 
            ack_timeout: Duration::from_millis(200), 
        }     
    }

    // Sends a message and retries until an ACK or ERR is received.
    pub async fn send_with_retry(&mut self, msg: &str) -> io::Result<()> {
        let text = msg.trim();
        println!("[ESP32_Handler] Sending message: '{}'", text);

        for attempt in 1..=self.max_retries {
            self.serial.send(&text).await?;

            // wait up to X seconds for a reply
            match timeout(self.ack_timeout, self.serial.read_line()).await {
                
                // Deal with String
                Ok(Ok(reply)) => {
                    println!("{}", reply);
                    let reply_trimmed = reply.trim();
                    if reply_trimmed == "ACK" {
                        println!("[ESP32_Handler] Got ACK on attempt {}", attempt);
                        return Ok(());
                    }
                    if reply_trimmed== "ERR" {
                        println!("[ESP32_Handler] Got ERR on attempt {}", attempt);
                        return Ok(());
                    }
                }
                Ok(Err(e)) => {
                    println!("[ESP32_Handler] Error reading reply: {}", e);
                }

                Err(_) => {
                    println!("[ESP32_Handler] Timeout waiting for reply on attempt {}", attempt);
                }
            }

            if attempt < self.max_retries {
                tokio::time::sleep(self.retry_delay).await;
                println!("[ESP32_Handler] 🔁 Retrying ({}/{})", attempt + 1, self.max_retries);
            }
           
        }

        Err(tokio::io::Error::new(tokio::io::ErrorKind::Other, "No ACK received"))
    }

    // Reads and parses an incoming message from the ESP32.
    pub async fn receive_message(&mut self) -> io::Result<EspMessage> {
        let raw = self.serial.read_line().await?;
        Ok(EspMessage::from_string(&raw).unwrap())
    }

    // Simple send without waiting for ACK
    pub async fn send_message(&mut self, msg: &str) -> io::Result<()> {
        println!("[ESP32_Handler] 📤 Sending (no wait): '{}'", msg);
        self.serial.send(msg).await
    }

}