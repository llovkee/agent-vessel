use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct ActionIntent {
    pub thought: String,
    pub tool: String,
    pub payload: String,
}

pub trait LlmAgent {
    fn generate_action(&mut self, goal: &str, observation: &str) -> impl Future<Output = ActionIntent> + Send;
}