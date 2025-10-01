use tokio_util::sync::CancellationToken;

pub struct SessionState {
    pub connected: bool,
    pub cancel_token: CancellationToken, // token per session
}

impl SessionState {
    pub fn new() -> Self {
        Self {
            connected: false,
            cancel_token: CancellationToken::new(),
        }
    }

    /// Reset session: cancel tasks and create new token
    pub fn reset(&mut self) {
        self.connected = false;
        self.cancel_token.cancel(); // cancel all session-scoped tasks
        self.cancel_token = CancellationToken::new(); // fresh token for next session
    }
}