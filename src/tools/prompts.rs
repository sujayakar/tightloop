use async_openai::types::{
    ChatCompletionTool,
    ChatCompletionToolType,
    FunctionObject,
};
use serde::Deserialize;
use serde_json::json;

const READ_FILE_PROMPT: &str = r#"
Reads a file from the local filesystem. You can access any file directly by using this tool.
If the User provides a path to a file assume that path is valid. It is okay to read a file that does not exist; an error will be returned.

Usage:
- You have the capability to call multiple tools in a single response. It is always better to speculatively read multiple files as a batch that are potentially useful.
- If you read a file that exists but has empty contents you will receive 'File is empty.'.
"#;

pub fn read_file_tool() -> ChatCompletionTool {
    ChatCompletionTool {
        r#type: ChatCompletionToolType::Function,
        function: FunctionObject {
            name: "read_file".to_string(),
            description: Some(READ_FILE_PROMPT.to_string()),
            parameters: Some(json!({
                "type": "object",
                "properties": {
                    "target_file": {
                        "type": "string",
                        "description": "The path of the file to read. You can use either a relative path in the workspace or an absolute path. If an absolute path is provided, it will be preserved as is."
                    }
                },
                "required": ["target_file"],
            })),
            strict: None,
        },
    }
}

#[derive(Debug, Deserialize)]
pub struct ReadFileArgs {
    pub target_file: String,
}

const LIST_DIR_PROMPT: &str = r#"
Lists files and directories in a given path. The 'target_directory' parameter can be relative to the workspace root or absolute.

Other details:
- The result does not display dot-files and dot-directories.
"#;

pub fn list_dir_tool() -> ChatCompletionTool {
    ChatCompletionTool {
        r#type: ChatCompletionToolType::Function,
        function: FunctionObject {
            name: "list_dir".to_string(),
            description: Some(LIST_DIR_PROMPT.to_string()),
            parameters: Some(json!({
                "type": "object",
                "properties": {
                    "target_directory": {
                        "type": "string",
                        "description": "Path to directory to list contents of."
                    }
                },
                "required": ["target_directory"],
            })),
            strict: None,
        },
    }
}

#[derive(Debug, Deserialize)]
pub struct ListDirArgs {
    pub target_directory: String,
}
