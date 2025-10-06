#[derive(Debug, Clone)]
pub enum ToolRequest {
    ToolCall { id: String, name: String, args: String },
}

#[derive(Debug, Clone)]
pub enum ToolResponse {
    ToolCallResult { id: String, result: Result<String, String> },
}
