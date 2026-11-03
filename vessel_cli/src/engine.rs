use vessel_core::agent::LlmAgent;
use vessel_core::state::{AgentState, Violation};
use vessel_proxy::ToolProxy;

pub struct ReactEngine<A: LlmAgent> {
    agent: A,
    proxy: ToolProxy,
    max_steps: u32
}

impl<A: LlmAgent> ReactEngine<A> {
    pub fn new(agent: A, proxy: ToolProxy, max_steps: u32) -> Self {
        Self {agent, proxy, max_steps}
    }

    pub async fn run(mut self, goal: &str) -> AgentState {
        let mut step = 0;
        let mut last_observation: String = "System initialized. Env ready".into();

        let mut caught_violation: Option<Violation> = None;

        while step < self.max_steps {
            let intent = self.agent.generate_action(goal, &last_observation).await;
            if intent.tool == "finish" {
                break;
            }

            match self.proxy.handle_request(&intent.tool, &intent.payload).await {
                Ok(res) => last_observation = format!("[Previous run]\nThought process: {}\nLast request result: {}", intent.thought, res),
                Err(violation) => {
                    caught_violation = Some(violation);
                    break;
                }
            }

            step += 1;
        }

        let final_progression = self.proxy.extract_progression();
        match caught_violation {
            Some(violation) => AgentState::Suspended(final_progression, violation),
            None => AgentState::Running(final_progression)
        }
    }
}