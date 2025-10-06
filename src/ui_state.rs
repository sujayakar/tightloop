use crate::types::PerformanceStats;

#[derive(Debug, Clone)]
pub enum GeneratingState {
    Idle,
    Generating,
}

#[derive(Debug, Clone)]
pub struct ChatUIState {
    messages: Vec<ChatUIMessage>,
    generating_state: GeneratingState,
    performance_stats: Option<PerformanceStats>,
}

impl ChatUIState {
    pub fn messages(&self) -> &[ChatUIMessage] {
        &self.messages
    }

    pub fn generating_state(&self) -> &GeneratingState {
        &self.generating_state
    }

    pub fn performance_stats(&self) -> &Option<PerformanceStats> {
        &self.performance_stats
    }
}

#[derive(Debug, Clone)]
pub enum ChatUIModification {
    AddUserMessage {
        text: String,
    },

    AddSystemMessage {
        text: String,
    },
    AppendSystemMessage {
        index: usize,
        text: String,
    },

    StartToolCall {
        name: String,
        args: String,
    },
    AppendToolCallArgs {
        index: usize,
        text: String,
    },
    StartToolCallExecution {
        index: usize,
    },
    CompleteToolCall {
        index: usize,
        result: Result<String, String>,
    },

    SetGeneratingState {
        state: GeneratingState,
    },

    SetPerformanceStats {
        stats: Option<PerformanceStats>,
    },
}

impl ChatUIState {
    pub fn new() -> Self {
        Self {
            messages: vec![],
            generating_state: GeneratingState::Idle,
            performance_stats: None,
        }
    }

    pub fn next_message_index(&self) -> usize {
        self.messages.len()
    }

    pub fn apply(&mut self, modification: ChatUIModification) -> anyhow::Result<()> {
        match modification {
            ChatUIModification::AddUserMessage { text } => {
                self.messages.push(ChatUIMessage::User(ChatUIUserMessage { text }));
            }
            ChatUIModification::AddSystemMessage { text } => {
                self.messages.push(ChatUIMessage::System(ChatUISystemMessage { text }));
            }
            ChatUIModification::AppendSystemMessage { index, text } => {
                let Some(ChatUIMessage::System(system_message)) = self.messages.get_mut(index) else {
                    return Err(anyhow::anyhow!("Message {index} is not a system message"));
                };
                system_message.text.push_str(&text);
            }
            ChatUIModification::StartToolCall { name, args } => {
                self.messages
                    .push(ChatUIMessage::ToolCall(ChatUIToolCall::Generating { name, args }));
            }
            ChatUIModification::AppendToolCallArgs { index, text } => {
                let Some(ChatUIMessage::ToolCall(ChatUIToolCall::Generating { args, .. })) =
                    self.messages.get_mut(index)
                else {
                    return Err(anyhow::anyhow!(
                        "Message {index} is not a currently generating tool call"
                    ));
                };
                args.push_str(&text);
            }
            ChatUIModification::StartToolCallExecution { index } => {
                let Some(ChatUIMessage::ToolCall(ChatUIToolCall::Generating { name, args })) =
                    self.messages.get_mut(index)
                else {
                    return Err(anyhow::anyhow!(
                        "Message {index} is not a currently generating tool call"
                    ));
                };
                self.messages[index] = ChatUIMessage::ToolCall(ChatUIToolCall::Executing {
                    name: name.clone(),
                    args: args.clone(),
                });
            }
            ChatUIModification::CompleteToolCall { index, result } => {
                let Some(ChatUIMessage::ToolCall(ChatUIToolCall::Executing { name, args })) =
                    self.messages.get_mut(index)
                else {
                    return Err(anyhow::anyhow!(
                        "Message {index} is not a currently executing tool call"
                    ));
                };
                self.messages[index] = ChatUIMessage::ToolCall(ChatUIToolCall::Complete {
                    name: name.clone(),
                    args: args.clone(),
                    result,
                });
            }
            ChatUIModification::SetGeneratingState { state } => {
                self.generating_state = state;
            }
            ChatUIModification::SetPerformanceStats { stats } => {
                self.performance_stats = stats;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub enum ChatUIMessage {
    User(ChatUIUserMessage),
    System(ChatUISystemMessage),
    ToolCall(ChatUIToolCall),
}

#[derive(Debug, Clone)]
pub struct ChatUIUserMessage {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct ChatUISystemMessage {
    pub text: String,
}

#[derive(Debug, Clone)]
pub enum ChatUIToolCall {
    Generating {
        name: String,
        args: String,
    },
    Executing {
        name: String,
        args: String,
    },
    Complete {
        name: String,
        args: String,
        result: Result<String, String>,
    },
}
