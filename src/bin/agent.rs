#![feature(try_blocks)]

use std::io::Stdout;

use agent::{
    server,
    tools,
    ui,
};
use async_openai::types::ReasoningEffort;
use clap::Parser;
use ratatui::prelude::CrosstermBackend;
use tokio::{
    fs,
    sync::mpsc,
    task::JoinSet,
};

#[derive(Parser)]
#[command(name = "agent")]
#[command(about = "A fast AI agent")]
struct Cli {
    /// The user prompt/query (optional)
    prompt: Option<String>,

    /// The LLM model to use
    #[arg(long)]
    model: String,

    /// The API key to use
    #[arg(long)]
    api_key: String,

    /// The base URL to use
    #[arg(long)]
    base_url: String,

    /// The reasoning effort level (low, medium, high)
    #[arg(long)]
    reasoning_effort: Option<String>,
}

async fn start_session(
    terminal: ratatui::Terminal<CrosstermBackend<Stdout>>,
    prompt: Option<String>,
    model: String,
    api_key: String,
    base_url: String,
    reasoning_effort: Option<ReasoningEffort>,
) -> anyhow::Result<()> {
    let (ui_tx, ui_rx) = mpsc::unbounded_channel();
    let (control_tx, control_rx) = mpsc::unbounded_channel();
    let (tool_req_tx, tool_req_rx) = mpsc::unbounded_channel();
    let (tool_resp_tx, tool_resp_rx) = mpsc::unbounded_channel();

    let mut join_set = JoinSet::new();
    join_set.spawn(ui::ui_loop(terminal, ui_rx, control_tx, prompt));
    join_set.spawn(server::server_loop(
        ui_tx,
        control_rx,
        tool_req_tx,
        tool_resp_rx,
        model,
        api_key,
        base_url,
        reasoning_effort,
    ));
    join_set.spawn(tools::executor::run_executor(tool_req_rx, tool_resp_tx));

    let first_result = join_set.join_next().await;
    if let Some(result) = first_result {
        if let Ok(Err(e)) = result {
            tracing::error!("Task failed: {e:?}");
        }
    }
    join_set.abort_all();
    while let Some(result) = join_set.join_next().await {
        if let Ok(Err(e)) = result {
            tracing::error!("Task failed: {e:?}");
        }
    }
    tracing::info!("Session ended");

    anyhow::Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let reasoning_effort = match cli.reasoning_effort {
        Some(effort) => match effort.as_str() {
            "low" => Some(ReasoningEffort::Low),
            "medium" => Some(ReasoningEffort::Medium),
            "high" => Some(ReasoningEffort::High),
            _ => anyhow::bail!("Invalid reasoning effort: {effort}"),
        },
        None => None,
    };

    let log_file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/agent.log")?;
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .with_writer(log_file)
        .init();

    if fs::try_exists(".env").await? {
        dotenvy::dotenv()?;
    }

    let terminal = ratatui::init();
    let result = start_session(
        terminal,
        cli.prompt,
        cli.model,
        cli.api_key,
        cli.base_url,
        reasoning_effort,
    )
    .await;
    ratatui::restore();

    result
}
