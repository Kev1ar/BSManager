use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Command {
    pub cmd: String,
    pub session_id: String,
    // pub MOTOR_ID: Option<u8>,
    // pub DIRECTION: String,
    // pub STEPS: Option<i32>,
}