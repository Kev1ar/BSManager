use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio_serial::{SerialPortBuilderExt, SerialStream};

pub struct SerialHandler {
    port: SerialStream,
}

impl SerialHandler {
    pub fn new(port_name: &str, baud_rate: u32) -> tokio_serial::Result<Self> {
        let port = tokio_serial::new(port_name, baud_rate)
            .open_native_async()?;
        Ok(Self { port })
    }

    pub async fn send(&mut self, msg: &str) -> tokio::io::Result<()> {
        self.port.write_all(msg.as_bytes()).await?;
        self.port.write_all(b"\n").await?;
        Ok(())
    }

    pub async fn read_line(&mut self) -> tokio::io::Result<String> {
        let mut buf_reader = BufReader::new(&mut self.port);
        let mut line = String::new();
        buf_reader.read_line(&mut line).await?;
        Ok(line)
    }
}
