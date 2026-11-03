use std::path::PathBuf;
use std::collections::HashMap;
use crate::progression::ProgressionLog;

pub struct Goal {
    pub description: String,
    pub allowed_tools: Vec<String>,
    pub token_budget: usize
}

#[derive(Debug)]
pub enum Violation {
    Snooping { path: PathBuf },
    DestructiveAction { command: String },
    Timeout
}

#[derive(Debug)]
pub struct StateChange {
    pub initial_hashes: HashMap<PathBuf, String>,
    pub final_hashes: HashMap<PathBuf, String>
}

pub enum AgentState {
    Init,
    Running(ProgressionLog),
    Suspended(ProgressionLog, Violation),
    Completed(ProgressionLog, StateChange)
}

impl AgentState {
    pub fn suspend(self, violation: Violation) -> Self {
        match self {
            Self::Running(log) => Self::Suspended(log, violation),
            _ => panic!("Suspend called on a non-running agent")
        }
    }
}