use tokio::fs;

pub const SYSTEM_PROMPT: &str = r#"
You are a powerful agentic AI coding assistant that optimizes for SPEED. Use tools as necessary but make
sure to run tools in parallel when possible. If you are unsure about the answer to the user's request,
gather more information by using additional tool calls or asking for clarification. Bias towards not asking
the user for help if you can find the answer yourself.
"#;

pub fn user_info() -> String {
    format!(
        r#"<user_info>
Arch: {arch}
OS: {os}
Shell: {shell}
Workspace Path: {workspace_path}
Note: Prefer using absolute paths over relative paths as tool call args when possible.
</user_info>
"#,
        arch = std::env::consts::ARCH,
        os = std::env::consts::OS,
        shell = std::env::var("SHELL").unwrap_or_else(|_| "unknown".to_string()),
        workspace_path = std::env::current_dir()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|_| "unknown".to_string()),
    )
}

pub const RULES: &str = r#"
Be sure to include language specifiers in Markdown code blocks.
"#;

pub async fn get_project_layout() -> anyhow::Result<String> {
    let current_dir = std::env::current_dir()?;

    let mut result = String::new();
    result.push_str(&format!(
        "<project_layout>\nBelow is a snapshot of the current workspace's file structure at the start of the \
         conversation. This snapshot will NOT update during the conversation.\n\n"
    ));
    result.push_str(&format!("{}\n", current_dir.display()));

    let root_metadata = fs::metadata(&current_dir).await?;
    let mut stack = vec![(0, current_dir, root_metadata)];
    while let Some((depth, path, metadata)) = stack.pop() {
        if depth > 0 {
            let space = "  ".repeat(depth);
            let suffix = if metadata.is_dir() {
                "/".to_string()
            } else {
                format!(" ({})", humansize::format_size(metadata.len(), humansize::DECIMAL))
            };
            if let Some(name) = path.file_name() {
                result.push_str(&format!(
                    "{space}- {name}{suffix}\n",
                    space = space,
                    name = name.display(),
                    suffix = suffix
                ));
            }
        }
        if metadata.is_dir() {
            let mut entries = fs::read_dir(&path).await?;
            let mut dir_entries = vec![];
            while let Some(entry) = entries.next_entry().await? {
                dir_entries.push(entry);
            }
            dir_entries.sort_by_key(|e| e.file_name());
            for entry in dir_entries.into_iter().rev() {
                let metadata = entry.metadata().await?;
                stack.push((depth + 1, entry.path(), metadata));
            }
        }
    }

    result.push_str(&format!("</project_layout>\n"));
    Ok(result)
}

#[tokio::test]
async fn test_get_project_layout() {
    let result = get_project_layout().await.unwrap();
    println!("{}", result);
}
