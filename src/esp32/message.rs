#[derive(Debug)]
pub struct EspMessage {
    pub cmd: String,
    pub motor: Option<u8>,
    pub steps: Option<u8>,
}

impl EspMessage {

    pub fn to_string(&self) -> String {
        format!(
            "{}:{}:{}",
            self.cmd,
            self.motor.unwrap_or(0),
            self.steps.unwrap_or(0)
        )
    }
    // take data string and convert into 
    pub fn from_string(data: &str) -> Option<EspMessage> {
        let parts: Vec<&str> = data.trim().split(':').collect();
        if parts.len() != 3 {
            return None;
        }
        Some(EspMessage {
            cmd: parts[0].to_string(),
            motor: parts[1].parse().ok(),
            steps: parts[2].parse().ok(),
        })
    }
}