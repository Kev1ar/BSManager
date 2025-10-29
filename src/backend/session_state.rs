use tokio_util::sync::CancellationToken;
use uuid::Uuid;

pub struct SessionState {
    pub connected: bool,
    pub cancel_token: CancellationToken, // token per session
    pub microscope_id: Option<Uuid>,   
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            connected: false,
            cancel_token: CancellationToken::new(),
            microscope_id: None,
        }
    }

    /// Reset session: cancel tasks and create new token
    pub fn reset(&mut self) {
        self.connected = false;
        self.cancel_token.cancel(); // cancel all session-scoped tasks
        self.cancel_token = CancellationToken::new(); // fresh token for next session
        self.microscope_id = None;   // clear microscope ID
    }
}