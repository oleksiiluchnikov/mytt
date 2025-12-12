use std::sync::mpsc::Sender;
use std::time::Duration;
use std::time::Instant;

pub struct Stopwatch {
    pub start_instant: Option<Instant>,
    pub elapsed: Duration,
}


pub struct CommandExecutor {
    pub sender: Sender<(String, String, String)>,
}
