use std::time::Duration;
use std::time::Instant;

use crate::engine::types::*;

impl Stopwatch {
    pub fn new() -> Self {
        Self {
            start_instant: None,
            elapsed: Duration::ZERO,
        }
    }

    pub fn start(&mut self) {
        if self.start_instant.is_none() {
            self.start_instant = Some(Instant::now());
        }
    }

    pub fn pause(&mut self) {
        if let Some(start) = self.start_instant {
            self.elapsed += start.elapsed();
            self.start_instant = None;
        }
    }

    pub fn resume(&mut self) {
        if self.start_instant.is_none() {
            self.start_instant = Some(Instant::now());
        }
    }

    pub fn stop(&mut self) {
        self.start_instant = None;
        self.elapsed = Duration::ZERO;
    }

    pub fn elapsed_ms(&self) -> u128 {
        match self.start_instant {
            Some(start) => (self.elapsed + start.elapsed()).as_millis() as u128,
            None => self.elapsed.as_millis() as u128,
        }
    }

    pub fn format_time(&self) -> String {
        let total_ms = self.elapsed_ms();
        let total_seconds = total_ms / 1000;

        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}
#[cfg(test)]
mod tests {
    use crate::engine::clock::Stopwatch;
    use std::thread::sleep;
    use std::time::{Duration, Instant};

    #[test]
    fn test_stopwatch() {
        let mut sw = Stopwatch::new();
        sw.start();
        sleep(Duration::from_millis(20));
        let t1 = sw.elapsed_ms();
        sleep(Duration::from_millis(20));
        let t2 = sw.elapsed_ms();
        assert!(t2 > t1);
    }

    #[test]
    fn elapsed_stops_when_paused() {
        let mut sw = Stopwatch::new();
        sw.start();
        sleep(Duration::from_millis(20));
        sw.pause();
        let t1 = sw.elapsed_ms();
        sleep(Duration::from_millis(20));
        let t2 = sw.elapsed_ms();
        assert_eq!(t1, t2);
    }

    #[test]
    fn elapsed_increases_while_running() {
        let mut sw = Stopwatch::new();
        sw.start();
        sleep(Duration::from_millis(20));
        let t1 = sw.elapsed_ms();
        sleep(Duration::from_millis(20));
        let t2 = sw.elapsed_ms();
        assert!(t2 > t1);
    }

    #[test]
    fn elapsed_continues_after_resume_without_jump() {
        let mut sw = Stopwatch::new();
        sw.start();
        sleep(Duration::from_millis(20));
        sw.pause();
        let paused = sw.elapsed_ms();
        sw.resume();
        sleep(Duration::from_millis(20));
        let resumed = sw.elapsed_ms();
        assert!(resumed > paused);
        assert!(resumed - paused < 50);
    }

    #[test]
    fn format_time_is_correct() {
        let mut sw = Stopwatch::new();
        sw.start();
        sleep(Duration::from_millis(1100));
        sw.pause();
        assert_eq!(sw.format_time(), "00:00:01");
    }
}
