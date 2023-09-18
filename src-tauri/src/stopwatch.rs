use std::time::{Instant, Duration};

pub struct Stopwatch {
    start_time: Option<Instant>,
    paused_duration: Duration,
}

impl Stopwatch {
   pub fn new() -> Self {
        Self { 
            start_time: None,
            paused_duration: Duration::new(0, 0),
        }
    }

    pub fn start(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now() - self.paused_duration);
            self.paused_duration = Duration::new(0, 0);
        }
    }

    pub fn pause(&mut self) {
        if let Some(start_time) = self.start_time {
            self.paused_duration = Instant::now() - start_time;
            self.start_time = None;
        }
    }

    pub fn resume(&mut self) {
        if self.start_time.is_none() {
            self.start_time = Some(Instant::now() - self.paused_duration);
        }
    }

    pub fn stop(&mut self) {
        self.start_time = None;
        self.paused_duration = Duration::new(0, 0);
    }

pub fn elapsed(&self) -> u128 {
    if let Some(start_time) = self.start_time {
        (self.paused_duration.as_millis() + (Instant::now() - start_time).as_millis()) as u128
    } else {
        self.paused_duration.as_millis() as u128
    }
}

pub fn format_time(&self) -> String {
    let elapsed = self.elapsed();
    let hours = elapsed / 3600000;
    let minutes = (elapsed % 3600000) / 60000;
    let seconds = (elapsed % 60000) / 1000;

    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

    pub fn reset(&mut self) {
        self.start_time = None;
        self.paused_duration = Duration::new(0, 0);
    }
}
