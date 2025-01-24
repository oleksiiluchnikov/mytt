// tests/stopwatch.rs
use std::thread::sleep;
use std::time::Duration;

#[cfg(test)]
mod stopwatch_tests {
    use crate::Stopwatch;

    #[test]
    fn test_start_and_stop() {
        let mut stopwatch = Stopwatch::new();
        stopwatch.start();
        sleep(Duration::from_millis(100));
        stopwatch.stop();
        assert!(stopwatch.elapsed() >= 100);
    }

    #[test]
    fn test_pause_and_resume() {
        let mut stopwatch = Stopwatch::new();
        stopwatch.start();
        sleep(Duration::from_millis(100));
        stopwatch.pause();
        let elapsed_after_pause = stopwatch.elapsed();
        sleep(Duration::from_millis(200));
        stopwatch.resume();
        sleep(Duration::from_millis(100));
        stopwatch.stop();
        assert_eq!(elapsed_after_pause, stopwatch.elapsed() - 300);
    }

    #[test]
    fn test_format_time() {
        let mut stopwatch = Stopwatch::new();
        stopwatch.start();
        sleep(Duration::from_secs(65));
        stopwatch.stop();
        assert_eq!(stopwatch.format_time(), "01:05:00");
    }

    #[test]
    fn test_reset() {
        let mut stopwatch = Stopwatch::new();
        stopwatch.start();
        sleep(Duration::from_millis(100));
        stopwatch.reset();
        assert_eq!(stopwatch.elapsed(), 0);
    }
}
