use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActionTrace {
    pub timestamp: DateTime<Utc>,
    pub tool_name: String,
    pub payload: String,
    pub exec_status: Result<String, String>,
}

#[derive(Debug, Clone, Default)]
pub struct ProgressionLog {
    pub actions: Vec<ActionTrace>
}

impl ProgressionLog {
    pub fn record_action(&mut self, action: ActionTrace) {
        self.actions.push(action);
    }
}