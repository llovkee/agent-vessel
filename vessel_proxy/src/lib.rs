use std::path::PathBuf;
use std::sync::Arc;
use chrono::Utc;
use vessel_core::state::Violation;
use vessel_core::progression::{ActionTrace, ProgressionLog};
use tokio::sync::Mutex;
use vessel_sandbox::SandboxManager;

pub struct ToolProxy {
    allowed_tools: Vec<String>,
    pub progression: ProgressionLog,
    sandbox: Arc<Mutex<SandboxManager>>,
}
impl ToolProxy {
    pub fn new(allowed_tools: Vec<String>, sandbox: Arc<Mutex<SandboxManager>>) -> Self {
        Self {
            allowed_tools,
            progression: ProgressionLog::default(),
            sandbox
        }
    }

    pub fn with_progression(allowed_tools: Vec<String>, progression: ProgressionLog, sandbox: Arc<Mutex<SandboxManager>>) -> Self {
        Self {
            allowed_tools,
            progression,
            sandbox
        }
    }

    pub async fn handle_request(&mut self, tool_name: &str, payload: &str) -> Result<String, Violation> {
        let timestamp = Utc::now();
        if !self.allowed_tools.contains(&tool_name.into()) {
            let err = format!("Unauthorised tool use '{}'", tool_name);
            self.progression.record_action(ActionTrace {
                timestamp,
                tool_name: tool_name.into(),
                payload: payload.into(),
                exec_status: Err(err.clone())
            });
            return Err(Violation::DestructiveAction {command: tool_name.into()});
        }

        if let Err(violation) = self.inspect_payload(tool_name, payload) {
            self.progression.record_action(ActionTrace {
                timestamp,
                tool_name: tool_name.into(),
                payload: payload.into(),
                exec_status: Err("Unsecure payload".into()),
            });
            return Err(violation);
        }

        let result = self.execute_tool(tool_name, payload).await;
        self.progression.record_action(ActionTrace {
            timestamp,
            tool_name: tool_name.into(),
            payload: payload.into(),
            exec_status: Ok(result.clone()),
        });

        Ok(result)
    }

    fn inspect_payload(&self, tool_name: &str, payload: &str) -> Result<(), Violation> {
        if tool_name == "execute_bash" { // TODO: global tool definition
            let forbidden = ["rm -rf", "mkfs", "curl", "wget", "/etc/passwd"];
            for f in forbidden {
                if payload.contains(f) {
                    return Err(Violation::DestructiveAction {command: payload.into()});
                }
            }

            if payload.contains("../") {
                return Err(Violation::Snooping {path: PathBuf::from(payload)});
            }
        }

        Ok(())
    }

    async fn execute_tool(&self, tool_name: &str, payload: &str) -> String {
        if tool_name == "execute_bash" {
            let sb = self.sandbox.lock().await;
            return sb.execute_bash(payload).await.unwrap_or_else(|err| format!("System error: {}", err))
        }

        "Tool execution not implemented".into()
    }

    pub fn extract_progression(self) -> ProgressionLog {
        self.progression
    }
}