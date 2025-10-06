use std::time::Duration;

use async_openai::types::{
    ChatCompletionRequestMessage,
    CreateChatCompletionRequestArgs,
    ReasoningEffort,
};
use futures::StreamExt;
use reqwest_eventsource::{
    Event,
    EventSource,
};
use tokio::{
    sync::mpsc,
    time::Instant,
};

use crate::{
    tools::prompts as tool_prompts,
    types::{
        FinishReason,
        PerformanceStats,
        Response,
        StreamResponse,
    },
};

pub struct LLMProvider {
    model: String,
    api_key: String,
    base_url: String,
    reasoning_effort: Option<ReasoningEffort>,
    http_client: reqwest::Client,
}

impl LLMProvider {
    pub fn new(
        model: String,
        api_key: String,
        base_url: String,
        reasoning_effort: Option<ReasoningEffort>,
    ) -> anyhow::Result<Self> {
        Ok(Self {
            model,
            api_key,
            base_url,
            reasoning_effort,
            http_client: reqwest::Client::new(),
        })
    }

    pub async fn chat(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
    ) -> mpsc::UnboundedReceiver<anyhow::Result<StreamChunk>> {
        let result: anyhow::Result<_> = try {
            let start = Instant::now();
            let mut args = CreateChatCompletionRequestArgs::default();

            if let Some(effort) = self.reasoning_effort.clone() {
                args.reasoning_effort(effort);
            }

            let args = args
                .model(&self.model)
                .messages(messages)
                .tools(vec![tool_prompts::read_file_tool(), tool_prompts::list_dir_tool()])
                .parallel_tool_calls(true)
                .stream(false)
                .build()?;
            let build_args = Instant::now();

            let url = format!("{}/chat/completions", self.base_url);
            let response = self
                .http_client
                .post(url)
                .json(&args)
                .header("Authorization", format!("Bearer {}", self.api_key))
                .send()
                .await?;
            let receive_headers = Instant::now();
            let status = response.status();
            if !status.is_success() {
                Err(anyhow::anyhow!("Failed to get stream: {}", status))?;
            }

            let response: Response = response.json().await?;
            let receive_body = Instant::now();

            tracing::info!("Client timeline:");
            tracing::info!("  Build args: {:?}", build_args - start);
            tracing::info!("  Receive headers: {:?}", receive_headers - start);
            tracing::info!("  Receive body: {:?}", receive_body - start);
            if let Some(time_info) = response.time_info {
                tracing::info!("Server latency:");
                tracing::info!("  Queue time: {:?}", Duration::from_secs_f64(time_info.queue_time));
                tracing::info!("  Prompt time: {:?}", Duration::from_secs_f64(time_info.prompt_time));
                tracing::info!(
                    "  Completion time: {:?}",
                    Duration::from_secs_f64(time_info.completion_time)
                );
                tracing::info!("  Total time: {:?}", Duration::from_secs_f64(time_info.total_time));
            }

            if let Some(usage) = response.usage {
                tracing::info!("Usage: {:?}", usage);
            }

            if response.choices.len() != 1 {
                Err(anyhow::anyhow!("Expected 1 choice, got {}", response.choices.len()))?;
            }
            let choice = response.choices[0].clone();
            let mut result = vec![];
            let mut total_bytes = 0;
            if let Some(content) = choice.message.content {
                total_bytes += content.len();
                result.push(StreamChunk::SystemMessage(content));
            }
            if let Some(tool_calls) = choice.message.tool_calls {
                for (i, tool_call) in tool_calls.into_iter().enumerate() {
                    total_bytes += tool_call.id.len();
                    let function = tool_call.function;
                    total_bytes += function.name.len();
                    total_bytes += function.arguments.len();
                    result.push(StreamChunk::StartToolCall {
                        index: i as u32,
                        id: tool_call.id,
                        name: function.name,
                    });
                    result.push(StreamChunk::AppendToolCallArgs {
                        index: i as u32,
                        text: function.arguments,
                    });
                }
            }
            if let Some(finish_reason) = choice.finish_reason {
                match finish_reason {
                    FinishReason::Stop => {}
                    FinishReason::ToolCalls => {}
                    _ => Err(anyhow::anyhow!("Unexpected finish reason: {:?}", finish_reason))?,
                }
            }
            tracing::info!("Total bytes: {total_bytes}");
            result
        };
        let (tx, rx) = mpsc::unbounded_channel();
        match result {
            Ok(result) => {
                for chunk in result {
                    let _ = tx.send(Ok(chunk));
                }
            }
            Err(e) => {
                let _ = tx.send(Err(e));
            }
        }
        rx
    }

    pub async fn stream(
        &self,
        messages: Vec<ChatCompletionRequestMessage>,
    ) -> mpsc::UnboundedReceiver<anyhow::Result<StreamChunk>> {
        let model = self.model.clone();
        let http_client = self.http_client.clone();
        let base_url = self.base_url.clone();
        let api_key = self.api_key.clone();
        let reasoning_effort = self.reasoning_effort.clone();

        let (tx, rx) = mpsc::unbounded_channel();
        let stream_generator = async move {
            let r: anyhow::Result<()> = try {
                tracing::debug!("Sending message: {:#?}", messages);
                let start = Instant::now();
                let mut args = CreateChatCompletionRequestArgs::default();
                args.model(&model)
                    .messages(messages)
                    .tools(vec![tool_prompts::read_file_tool(), tool_prompts::list_dir_tool()])
                    .parallel_tool_calls(true)
                    .stream(true);
                if let Some(effort) = reasoning_effort {
                    args.reasoning_effort(effort);
                }
                let args = args.build()?;
                let build_args = Instant::now();

                let url = format!("{}/chat/completions", base_url);
                let request_builder = http_client
                    .post(url)
                    .json(&args)
                    .header("Authorization", format!("Bearer {}", api_key));

                let sse = EventSource::new(request_builder)?;
                tokio::pin!(sse);

                let mut chunk_timestamps = vec![];
                let mut usage = None;
                let mut time_info = None;

                while let Some(event_r) = sse.next().await {
                    let mut useful_bytes = 0;
                    let message = match event_r {
                        Ok(Event::Message(message)) => message,
                        Ok(_) => continue,
                        Err(e) => {
                            if let reqwest_eventsource::Error::InvalidStatusCode(_, response) = e {
                                let status = response.status();
                                let text = response.text().await?;
                                Err(anyhow::anyhow!("Invalid status code: {status}: {text}"))?;
                                unreachable!();
                            }
                            Err(e)?;
                            unreachable!();
                        }
                    };
                    tracing::debug!("Received event: {}", message.data);
                    let mut resp: StreamResponse = serde_json::from_str(&message.data)?;
                    if let Some(u) = resp.usage {
                        usage = Some(u);
                    }
                    if let Some(t) = resp.time_info {
                        time_info = Some(t);
                    }
                    if resp.choices.len() != 1 {
                        Err(anyhow::anyhow!("Expected 1 choice, got {}", resp.choices.len()))?;
                    }
                    let choice = resp.choices.remove(0);
                    if let Some(content) = choice.delta.content {
                        useful_bytes += content.len();
                        tx.send(Ok(StreamChunk::SystemMessage(content)))?;
                    }
                    if let Some(reasoning) = choice.delta.reasoning {
                        useful_bytes += reasoning.len();
                        tx.send(Ok(StreamChunk::SystemMessage(reasoning)))?;
                    }
                    if let Some(tool_calls) = choice.delta.tool_calls {
                        for tool_call in tool_calls {
                            let function = tool_call
                                .function
                                .ok_or_else(|| anyhow::anyhow!("Tool call function is missing"))?;
                            if let Some(id) = tool_call.id
                                && !id.is_empty()
                            {
                                let chunk = StreamChunk::StartToolCall {
                                    index: tool_call.index,
                                    id,
                                    name: function
                                        .name
                                        .ok_or_else(|| anyhow::anyhow!("Tool call name is missing"))?,
                                };
                                tx.send(Ok(chunk))?;
                            }
                            if let Some(args) = function.arguments {
                                useful_bytes += args.len();
                                let chunk = StreamChunk::AppendToolCallArgs {
                                    index: tool_call.index,
                                    text: args,
                                };
                                tx.send(Ok(chunk))?;
                            }
                        }
                    }
                    chunk_timestamps.push((Instant::now(), useful_bytes));
                    if let Some(finish_reason) = choice.finish_reason {
                        match finish_reason {
                            FinishReason::Stop | FinishReason::ToolCalls => {
                                tracing::info!("Stopping stream because of finish reason: {:?}", finish_reason);
                                break;
                            }
                            _ => Err(anyhow::anyhow!("Unexpected finish reason: {:?}", finish_reason))?,
                        }
                    }
                }
                tracing::debug!("Client timeline:");
                tracing::debug!("  Build args: {:?}", build_args - start);
                for (i, (timestamp, bytes)) in chunk_timestamps.iter().enumerate() {
                    tracing::debug!("  Chunk {i} ({:?} bytes): {:?}", bytes, *timestamp - start);
                }
                if let Some(time_info) = time_info {
                    tracing::info!("Server latency:");
                    tracing::info!("  Queue time: {:?}", Duration::from_secs_f64(time_info.queue_time));
                    tracing::info!("  Prompt time: {:?}", Duration::from_secs_f64(time_info.prompt_time));
                    tracing::info!(
                        "  Completion time: {:?}",
                        Duration::from_secs_f64(time_info.completion_time)
                    );
                    tracing::info!("  Total time: {:?}", Duration::from_secs_f64(time_info.total_time));
                }
                if let Some(usage) = usage {
                    tracing::info!("Usage: {:#?}", usage);
                }
                if !chunk_timestamps.is_empty() {
                    let ttft = chunk_timestamps[0].0 - start;

                    let all_bytes = chunk_timestamps[1..].iter().map(|(_, bytes)| bytes).sum::<usize>();
                    let req_duration = chunk_timestamps[chunk_timestamps.len() - 1].0 - start;
                    let bytes_per_sec = all_bytes as f64 / req_duration.as_secs_f64();
                    let stats = PerformanceStats { ttft, bytes_per_sec };
                    tracing::info!("Performance stats: {:?}", stats);
                    tx.send(Ok(StreamChunk::PerformanceStats(stats)))?;
                }
            };
            if let Err(e) = r {
                let _ = tx.send(Err(e));
            }
        };
        tokio::spawn(stream_generator);
        rx
    }
}

#[derive(Debug)]
pub enum StreamChunk {
    SystemMessage(String),
    StartToolCall { index: u32, id: String, name: String },
    AppendToolCallArgs { index: u32, text: String },
    PerformanceStats(PerformanceStats),
}
