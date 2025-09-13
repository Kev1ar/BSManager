use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    pub CMD: String,
    pub AI_ON: Option<bool>,
    pub MOTOR_ID: Option<u8>,
    pub STEPS: Option<[i32; 2]>,
    pub META: Option<serde_json::Value>,
}
