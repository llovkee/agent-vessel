mod evaluator;
mod engine;
mod ollama;
mod demo;

use std::sync::Arc;
use vessel_core::state::{AgentState};
use clap::Parser;
use vessel_core::state::{Goal, StateChange};
use vessel_monitor::WorkspaceMonitor;
use vessel_sandbox::SandboxManager;
use tokio::sync::Mutex;
use vessel_proxy::ToolProxy;
use crate::demo::DemoHackerAgent;
use crate::evaluator::Evaluator;

#[derive(Parser, Debug)]
#[command(author, version, about = "Agent Vessel")]
struct Args {
    #[arg(long, default_value = "demo")]
    mode: String, // "ollama" or "demo"

    #[arg(short, long, default_value = "gemma4")]
    model: String, // ollama model

    #[arg(short, long)]
    goal: String,

    #[arg(short, long, help = "Define filesystem for agent. Will be deep-copied.")]
    filesystem: String,
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    let workspace = std::env::current_dir()?.join("workspace_tmp");
    let _ = std::fs::remove_dir_all(&workspace);
    std::fs::create_dir_all(&workspace)?;
    let filesystem = std::fs::canonicalize(&args.filesystem)?;
    let mut options = fs_extra::dir::CopyOptions::new();
    options.copy_inside = true;
    fs_extra::dir::copy(&filesystem, &workspace, &options)?;

    let goal = Goal {
        description: args.goal,
        allowed_tools: vec!["write_file".into(), "execute_bash".into(), "finish".into()],
        token_budget: 4096
    };

    let monitor = WorkspaceMonitor::new(&workspace);
    let initial_hashes = monitor.snapshot()?;

    let mut sandbox = SandboxManager::new()?;
    sandbox.spawn("ubuntu:latest", &workspace).await?;
    let shared_sb = Arc::new(Mutex::new(sandbox));

    let proxy = ToolProxy::new(
        goal.allowed_tools.clone(),
        shared_sb.clone(),
    );

    println!("Running {} agent", args.mode);
    let final_state = match args.mode.as_str() {
        "ollama" => {
            let agent = ollama::OllamaAgent::new(&args.model);
            let engine = engine::ReactEngine::new(agent, proxy, 3);
            engine.run(&goal.description).await
        }
        "demo" => {
            let agent = DemoHackerAgent { step: 0 };
            let engine = engine::ReactEngine::new(agent, proxy, 3);
            engine.run(&goal.description).await
        }
        _ => panic!("Unsupported mode: {}", args.mode)
    };

    {
        let mut sb = shared_sb.lock().await;
        sb.purge().await?;
    }


    let evaluated_state = match final_state {
        AgentState::Suspended(log, violation) => {
            println!("  [System Interrupted] Agent suspended");
            println!("      Security rules violated: {:?}", violation);
            AgentState::Suspended(log, violation)
        }
        AgentState::Running(log) => {
            println!("  [Finish] Agent finished task");
            println!("  [Logs] {:?}", log);
            let final_hashes = monitor.snapshot()?;
            let changes = StateChange {initial_hashes, final_hashes};
            AgentState::Completed(log, changes)
        }
        _ => unreachable!(),
    };

    match Evaluator::evaluate(evaluated_state, &goal) {
        Ok(score) => {
            println!("  [Evaluated] Agent's score: {}", score);
        }
        Err(err) => {
            eprintln!("  [Evaluator Error] Agent is in state which cannot be evaluated: {}", err);
        }
    }

    Ok(())
}
