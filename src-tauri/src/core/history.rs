use crate::core::domain::FlowRating;
use chrono::{DateTime, Utc}; // Change Local to Utc
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub id: String,
    pub start_time: DateTime<Utc>, // <-- Use Utc here
    pub planned_duration_s: u32,
    pub actual_duration_ms: u128,
    pub focus_rating: FlowRating,
    pub phase: String, // "work", "shortBreak", "longBreak"
    pub break_taken: bool,
    pub break_duration_s: Option<u32>,
}

impl SessionRecord {
    pub fn new(
        planned_duration_s: u32,
        actual_duration_ms: u128,
        focus_rating: FlowRating,
        phase: String,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            start_time: chrono::Utc::now(), // <-- Use Utc here
            planned_duration_s,
            actual_duration_ms,
            focus_rating,
            phase,
            break_taken: false,
            break_duration_s: None,
        }
    }

    pub fn actual_duration_s(&self) -> u32 {
        (self.actual_duration_ms / 1000) as u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{DateTime, Utc};

    #[test]
    fn test_serialization() {
        let record = SessionRecord::new(900, 900_000, FlowRating::Flow, "work".to_string());

        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("\"start_time\":"));
        assert!(json.contains("\"planned_duration_s\":900"));
        assert!(json.contains("\"actual_duration_ms\":900000"));
        assert!(json.contains("\"focus_rating\":\"flow\""));
        assert!(json.contains("\"phase\":\"work\""));
        assert!(json.contains("\"break_taken\":false"));
        assert!(json.contains("\"break_duration_s\":null"));
    }
}
