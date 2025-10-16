use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Command {
    #[serde(rename = "type")]
    pub cmd: String,
    // pub session_id: String,
    // pub ai_on: Option<bool>,
    // pub motor_id: Option<u8>,
    // pub steps: Option<[i32; 2]>,
    // pub meta: Option<serde_json::Value>,
}
