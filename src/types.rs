

#[derive(Clone)]
pub struct Progress {
    pub success: u32,
    pub failed: u32,
    pub total: u32,
}

impl Progress {
    pub fn new(total: u32) -> Self {
        Self {
            success: 0,
            failed: 0,
            total,
        }
    }

    pub fn increment_success(&mut self) {
        self.success += 1;
    }

    pub fn increment_failed(&mut self) {
        self.failed += 1;
    }
}

pub enum Message {
    Warning(String),
    Failed(String),
    Message(String),
    Progress(Progress),
    Completed,
}
