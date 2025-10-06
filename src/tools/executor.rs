use tokio::{
    fs,
    sync::mpsc,
};

use crate::tools::{
    prompts::{
        ListDirArgs,
        ReadFileArgs,
    },
    protocol::{
        ToolRequest,
        ToolResponse,
    },
};

pub async fn run_executor(
    mut requests: mpsc::UnboundedReceiver<ToolRequest>,
    responses: mpsc::UnboundedSender<ToolResponse>,
) -> anyhow::Result<()> {
    while let Some(request) = requests.recv().await {
        let start = tokio::time::Instant::now();
        let ToolRequest::ToolCall { id, name, args } = request;
        tracing::info!("Executing tool {name} (id: {id})");
        tracing::debug!("  {args}");
        let result = execute_tool(name, args).await;
        let response = ToolResponse::ToolCallResult {
            id,
            result: result.map_err(|e| e.to_string()),
        };
        tracing::info!("Finished in {:?}", start.elapsed());
        tracing::debug!("  {response:?}");
        responses.send(response)?;
    }
    Ok(())
}

async fn execute_tool(name: String, args: String) -> anyhow::Result<String> {
    match name.as_str() {
        "list_dir" => {
            let args: ListDirArgs = serde_json::from_str(&args)?;
            let mut entries = fs::read_dir(&args.target_directory).await?;
            let mut result = String::new();
            result.push_str(&args.target_directory);
            result.push_str(":\n");
            while let Some(entry) = entries.next_entry().await? {
                result.push_str("  ");
                result.push_str(
                    &entry
                        .file_name()
                        .into_string()
                        .map_err(|_| anyhow::anyhow!("Failed to convert file name to string"))?,
                );
                let file_type = entry.file_type().await?;
                result.push_str(&format!(" ({:?})\n", file_type));
            }
            Ok(result)
        }
        "read_file" => {
            let args: ReadFileArgs = serde_json::from_str(&args)?;
            let mut contents = fs::read_to_string(&args.target_file).await?;
            if contents.is_empty() {
                contents = "File is empty.".to_string();
            }
            Ok(contents)
        }
        _ => anyhow::bail!("Unknown tool: {name}"),
    }
}
