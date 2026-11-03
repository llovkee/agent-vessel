use reqwest::Client;
use vessel_core::agent::{ActionIntent, LlmAgent};
use serde_json::json;

pub struct OllamaAgent {
    client: Client,
    model: String,
    api_url: String,
    system_prompt: String,
}

impl OllamaAgent {
    pub fn new(model: &str) -> Self {
        let system_prompt = r#"You are an agent executing tasks on a filesystem.
Available tools: 'execute_bash' -> execute bash command command
                 'finish' -> when you think user goal have been met
You must ALWAYS respond in JSON format matching the following schema:
{{
  "thought": "Your reasoning",
  "tool": "tool_name",
  "payload": "arguments"
}}
            "#.to_string();
        Self {
            client: Client::new(),
            model: model.into(),
            api_url: "http://localhost:11434/api/generate".into(),
            system_prompt
        }
    }
}

impl LlmAgent for OllamaAgent {
    async fn generate_action(&mut self, goal: &str, observation: &str) -> ActionIntent {
        let prompt = format!("User goal: {}\nLast observation: {}\nGenerate action (JSON):", goal, observation);

        let body = json!({
            "model": self.model,
            "system": self.system_prompt,
            "prompt": prompt,
            "format": "json",
            "stream": false
        });

        let response = self.client
            .post(&self.api_url)
            .json(&body)
            .send()
            .await
            .expect("Error while communicating with ollama");

        let res_json: serde_json::Value = response.json().await.expect("Invalid response format");
        let response_text = res_json["response"].as_str().unwrap_or("{}");

        serde_json::from_str(response_text).unwrap_or_else(|_| ActionIntent {
            thought: "Fallback error parsing".into(),
            tool: "finish".into(),
            payload: "".into(),
        })
    }
}