use std::{
    collections::HashMap,
    time::Duration,
};

use async_openai::types::{
    ChatCompletionMessageToolCall,
    ChatCompletionRequestAssistantMessage,
    ChatCompletionRequestAssistantMessageContent,
    ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent,
    ChatCompletionRequestToolMessage,
    ChatCompletionRequestToolMessageContent,
    ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent,
    ChatCompletionToolType,
    FunctionCall,
    ReasoningEffort,
};
use tokio::{
    sync::{
        mpsc,
        oneshot,
    },
    time::Instant,
};

use crate::{
    control::ControlMessage,
    llm_provider::{
        LLMProvider,
        StreamChunk,
    },
    prompts,
    tools::protocol::{
        ToolRequest,
        ToolResponse,
    },
    ui_state::{
        ChatUIModification,
        ChatUIState,
        GeneratingState,
    },
};

pub async fn server_loop(
    ui_tx: mpsc::UnboundedSender<ChatUIModification>,
    mut control_rx: mpsc::UnboundedReceiver<ControlMessage>,
    tool_req_tx: mpsc::UnboundedSender<ToolRequest>,
    mut tool_resp_rx: mpsc::UnboundedReceiver<ToolResponse>,
    model: String,
    api_key: String,
    base_url: String,
    reasoning_effort: Option<ReasoningEffort>,
) -> anyhow::Result<()> {
    let llm_provider = LLMProvider::new(model, api_key, base_url, reasoning_effort)?;
    let ui_state = ChatUIState::new();
    let mut ui_batcher = UIBatcher::new(ui_tx, ui_state);

    let mut messages = vec![
        ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
            content: ChatCompletionRequestSystemMessageContent::Text(prompts::SYSTEM_PROMPT.to_string()),
            name: None,
        }),
        ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Text(prompts::user_info()),
            name: None,
        }),
        ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Text(prompts::RULES.to_string()),
            name: None,
        }),
        ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Text(prompts::get_project_layout().await?),
            name: None,
        }),
    ];

    let mut last_request_start: Option<tokio::time::Instant> = None;

    'shutdown: loop {
        let Some(ControlMessage::UserMessage(user_message)) = control_rx.recv().await else {
            break;
        };

        messages.push(ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Text(user_message.clone()),
            name: None,
        }));
        let modification = ChatUIModification::AddUserMessage {
            text: user_message.clone(),
        };
        ui_batcher.apply(modification)?;

        let mut in_progress_tool_calls = HashMap::new();

        loop {
            while !in_progress_tool_calls.is_empty() {
                let Some(ToolResponse::ToolCallResult { id, result }) = tool_resp_rx.recv().await else {
                    break 'shutdown;
                };
                let Some(message_index) = in_progress_tool_calls.remove(&id) else {
                    anyhow::bail!("Tool call {id} is not in progress");
                };
                let formatted_result = match result {
                    Ok(ref result) => result.clone(),
                    Err(ref error) => format!("Error: {error}"),
                };
                messages.push(ChatCompletionRequestMessage::Tool(ChatCompletionRequestToolMessage {
                    content: ChatCompletionRequestToolMessageContent::Text(formatted_result),
                    tool_call_id: id.clone(),
                }));
                let modification = ChatUIModification::CompleteToolCall {
                    index: message_index,
                    result,
                };
                ui_batcher.apply(modification.clone())?;
            }

            tracing::info!("Streaming LLM response");
            for (i, message) in messages.iter().enumerate() {
                let mut message_str = format!("{message:?}");
                if message_str.len() > 100 {
                    message_str.truncate(97);
                    message_str.push_str("...");
                }
                tracing::info!("  {i}: {message_str}");
            }

            if let Some(last_request_start) = last_request_start {
                let elapsed = last_request_start.elapsed();
                tracing::info!("{elapsed:?} since last request");
            }
            let request_start = tokio::time::Instant::now();
            last_request_start = Some(request_start);

            // Set generating state to Generating
            let modification = ChatUIModification::SetGeneratingState {
                state: GeneratingState::Generating,
            };
            ui_batcher.apply(modification)?;

            let stream = llm_provider.stream(messages.clone()).await;
            tokio::pin!(stream);

            let mut current_system_message_index = None;
            let mut current_system_message_text = String::new();

            let mut streaming_tool_calls = HashMap::new();

            #[allow(unused)]
            struct StreamingToolCall {
                index: u32,
                id: String,
                name: String,
                args: String,

                ui_index: usize,
            }

            while let Some(chunk_r) = stream.recv().await {
                let chunk = chunk_r?;
                tracing::debug!("Received chunk: {:?}", chunk);
                match chunk {
                    StreamChunk::SystemMessage(text) => {
                        current_system_message_text.push_str(&text);
                        match current_system_message_index {
                            Some(index) => {
                                let modification = ChatUIModification::AppendSystemMessage { index, text };
                                ui_batcher.apply(modification.clone())?;
                            }
                            None => {
                                let index = ui_batcher.ui_state.next_message_index();
                                let modification = ChatUIModification::AddSystemMessage { text };
                                ui_batcher.apply(modification.clone())?;
                                current_system_message_index = Some(index);
                            }
                        }
                    }
                    StreamChunk::StartToolCall { index, id, name } => {
                        anyhow::ensure!(
                            !in_progress_tool_calls.contains_key(&id),
                            "Tool call {id} is already in progress"
                        );
                        let ui_index = ui_batcher.ui_state.next_message_index();
                        streaming_tool_calls.insert(
                            index,
                            StreamingToolCall {
                                index,
                                id,
                                name: name.clone(),
                                args: String::new(),
                                ui_index,
                            },
                        );
                        let modification = ChatUIModification::StartToolCall {
                            name,
                            args: String::new(),
                        };
                        ui_batcher.apply(modification.clone())?;
                    }
                    StreamChunk::AppendToolCallArgs { index, text } => {
                        let Some(StreamingToolCall { args, ui_index, .. }) = streaming_tool_calls.get_mut(&index)
                        else {
                            anyhow::bail!("Tool call {index} not currently streaming");
                        };
                        args.push_str(&text);

                        let modification = ChatUIModification::AppendToolCallArgs { index: *ui_index, text };
                        ui_batcher.apply(modification.clone())?;
                    }
                    StreamChunk::PerformanceStats(stats) => {
                        let modification = ChatUIModification::SetPerformanceStats { stats: Some(stats) };
                        ui_batcher.apply(modification)?;
                    }
                }
            }

            tracing::info!(
                "Stream ended. Current system message length: {}, Tool calls in progress: {}",
                current_system_message_text.len(),
                in_progress_tool_calls.len()
            );

            let mut tool_calls = vec![];
            for (_, tool_call) in streaming_tool_calls {
                let modification = ChatUIModification::StartToolCallExecution {
                    index: tool_call.ui_index,
                };
                ui_batcher.apply(modification.clone())?;
                in_progress_tool_calls.insert(tool_call.id.clone(), tool_call.ui_index);
                tool_req_tx.send(ToolRequest::ToolCall {
                    id: tool_call.id.clone(),
                    name: tool_call.name.clone(),
                    args: tool_call.args.clone(),
                })?;
                tool_calls.push(ChatCompletionMessageToolCall {
                    id: tool_call.id,
                    r#type: ChatCompletionToolType::Function,
                    function: FunctionCall {
                        name: tool_call.name,
                        arguments: tool_call.args,
                    },
                })
            }
            if !current_system_message_text.is_empty() || !tool_calls.is_empty() {
                let content = if !current_system_message_text.is_empty() {
                    Some(ChatCompletionRequestAssistantMessageContent::Text(
                        current_system_message_text,
                    ))
                } else {
                    None
                };
                let tool_calls = if !tool_calls.is_empty() { Some(tool_calls) } else { None };
                messages.push(ChatCompletionRequestMessage::Assistant(
                    ChatCompletionRequestAssistantMessage {
                        content,
                        refusal: None,
                        name: None,
                        audio: None,
                        tool_calls,
                        #[allow(deprecated)]
                        function_call: None,
                    },
                ))
            }

            // Set generating state back to Idle
            let modification = ChatUIModification::SetGeneratingState {
                state: GeneratingState::Idle,
            };
            ui_batcher.apply(modification)?;

            if in_progress_tool_calls.is_empty() {
                break;
            }
        }
    }

    anyhow::Ok(())
}

struct UIBatcher {
    _sender: tokio::task::JoinHandle<anyhow::Result<()>>,
    _shutdown_tx: oneshot::Sender<()>,
    modifications_tx: mpsc::UnboundedSender<ChatUIModification>,
    ui_state: ChatUIState,
}

impl UIBatcher {
    fn new(ui_tx: mpsc::UnboundedSender<ChatUIModification>, ui_state: ChatUIState) -> Self {
        let (modifications_tx, modifications_rx) = mpsc::unbounded_channel();
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        let worker_future = Self::go(Duration::from_millis(100), modifications_rx, ui_tx, shutdown_rx);
        let sender = tokio::spawn(worker_future);
        Self {
            _sender: sender,
            _shutdown_tx: shutdown_tx,
            modifications_tx,
            ui_state,
        }
    }

    async fn go(
        batch_interval: Duration,
        mut modifications_rx: mpsc::UnboundedReceiver<ChatUIModification>,
        ui_tx: mpsc::UnboundedSender<ChatUIModification>,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) -> anyhow::Result<()> {
        let mut deferred_modifications: Vec<ChatUIModification> = Vec::new();
        let mut last_sent = Instant::now() - batch_interval;

        loop {
            let now = Instant::now();
            let is_deferred = !deferred_modifications.is_empty();
            tokio::select! {
                _ = tokio::time::sleep_until(last_sent + batch_interval), if is_deferred => {
                    for modification in deferred_modifications.drain(..) {
                        ui_tx.send(modification)?;
                    }
                    last_sent = now;
                }
                modification = modifications_rx.recv() => {
                    let Some(modification) = modification else {
                        break;
                    };
                    Self::merge_modifications(&mut deferred_modifications, modification);
                }
                _ = &mut shutdown_rx => {
                    break;
                }
            }
        }
        if !deferred_modifications.is_empty() {
            for modification in deferred_modifications.drain(..) {
                ui_tx.send(modification)?;
            }
        }
        anyhow::Ok(())
    }

    fn merge_modifications(deferred_modifications: &mut Vec<ChatUIModification>, modification: ChatUIModification) {
        if let ChatUIModification::AppendSystemMessage { text: ref new_text, .. } = modification {
            match deferred_modifications.last_mut() {
                Some(ChatUIModification::AddSystemMessage { text }) => {
                    text.push_str(&new_text);
                    return;
                }
                Some(ChatUIModification::AppendSystemMessage { text, .. }) => {
                    text.push_str(&new_text);
                    return;
                }
                _ => (),
            }
        }
        deferred_modifications.push(modification);
    }

    fn apply(&mut self, modification: ChatUIModification) -> anyhow::Result<()> {
        self.ui_state.apply(modification.clone())?;
        self.modifications_tx.send(modification)?;
        Ok(())
    }
}
