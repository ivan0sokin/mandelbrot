use std::time::Duration;

pub struct RenderInfo {
    pub elapsed: Duration,
    pub bytes: usize
}

impl RenderInfo {
    pub fn new(elapsed: Duration, bytes: usize) -> Self {
        Self {
            elapsed,
            bytes
        }
    }
}