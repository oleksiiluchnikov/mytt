// tests/command_executor.rs
use std::process::Command;
use std::thread::sleep;
use std::time::Duration;

#[cfg(test)]
mod command_executor_tests {
    use crate::CommandExecutor;
    use std::path::PathBuf;

    #[test]
    fn test_execute() {
        let executor = CommandExecutor::new();
        let script_path = PathBuf::from("./tests/test_script.sh");
        executor.execute(
            script_path.clone(),
            "00:00:05".to_string(),
            "Test App".to_string(),
        );
        sleep(Duration::from_secs(6));
        let output = Command::new("sh")
            .arg("-c")
            .arg("cat ./tests/test_output.txt")
            .output()
            .expect("Failed to execute command");
        let output_str = String::from_utf8_lossy(&output.stdout);
        assert_eq!(output_str.trim(), "Test App 00:00:05");
    }
}
