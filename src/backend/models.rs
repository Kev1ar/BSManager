use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Command {
    #[serde(rename = "welcome")] 
    Welcome,
    Move { direction: String },           // e.g., "UP", "DOWN", "LEFT", "RIGHT"
    Capture,                              // capture a single image
    StartStream,                          // start live streaming
    StopStream,                           // stop live streaming
    SetMicroscope { microscope_id: Uuid }, // assign microscope ID
    #[serde(rename = "heartbeat")] 
    Heartbeat,                            // heartbeat signal
    Shutdown,                             // shutdown device
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Response {
    Ack { command: String },              // acknowledgment of command
    Error { message: String },            // error message
    ImageCaptured {
        image_data: String,               // Base64 encoded image
        format: String,                   // e.g., "jpg", "png"
    },
    StreamFrame {
        frame_data: String,               // Base64 encoded frame
        format: String,                   // e.g., "jpg"
        timestamp: i64,                   // Unix timestamp in milliseconds
    },
    StreamStarted,
    StreamStopped,
    Status { status: String },            // e.g., "Idle", "Moving", etc.
    Heartbeat { heartbeat: String },      // e.g., "alive"
}