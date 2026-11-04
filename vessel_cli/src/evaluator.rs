use vessel_core::progression::ProgressionLog;
use vessel_core::state::{AgentState, Goal, StateChange};

pub struct Evaluator;

impl Evaluator {
    pub fn evaluate(state: AgentState, goal: &Goal) -> Result<f32, &'static str> {
        match state {
            AgentState::Suspended(_log, _violation) => {
                Ok(0f32)
            },
            AgentState::Completed(log, change) => Self::verify_changes(&change, goal, &log),
            _ => Err("Agent state is in invalid state for evaluation"),
        }
    }

    fn verify_changes(change: &StateChange, _goal: &Goal, _log: &ProgressionLog) -> Result<f32, &'static str> {
        let mut score = 1f32;
        let mut actual_changes = Vec::new();

        for (path, init_hash) in &change.initial_hashes {
           if let Some(final_hash) = change.final_hashes.get(path) {
               if init_hash != final_hash {
                   actual_changes.push(path.clone())
               }
           } else { // File was deleted
               score *= 0.9;
           }
        }

        for path in change.final_hashes.keys() {
            if !change.initial_hashes.contains_key(path) {
                actual_changes.push(path.clone()) // File was added
            }
        }

        for changed_path in &actual_changes {
            if changed_path.starts_with("tests") {
                return Ok(0f32); // Tests were overwritten
            }
        }

        // claims did something, but did not

        Ok(score)
    }
}