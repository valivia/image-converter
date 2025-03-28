use std::{path::PathBuf, time::Duration};

pub enum Update {
    Message(String),
    StartProcessing(PathBuf),
    FinishedProcessing(PathBuf, bool, Duration),
    QueueCompleted(Duration),
}
