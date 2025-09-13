struct MockEspHandler;

impl MockEspHandler {
    async fn send_with_retry(&mut self, msg: &EspMessage) -> Result<(), String> {
        println!("(MOCK) Sending: {:?}", msg);
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        println!("(MOCK) Reply received!");
        Ok(())
    }
}
