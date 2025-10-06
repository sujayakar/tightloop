use std::io::Stdout;

use crossterm::event::{
    Event,
    EventStream,
    KeyCode,
    KeyEventKind,
};
use futures::StreamExt;
use humansize::{
    DECIMAL,
    format_size,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::CrosstermBackend,
    style::Stylize,
    symbols::border,
    text::{
        Line,
        Text,
    },
    widgets::{
        Block,
        Paragraph,
        Widget,
        Wrap,
    },
};
use tokio::sync::mpsc;

use crate::{
    control::ControlMessage,
    markdown_render::render_markdown_text,
    ui_state::{
        ChatUIMessage,
        ChatUIModification,
        ChatUIState,
        ChatUIToolCall,
        GeneratingState,
    },
};

struct UIState {
    chat: ChatUIState,
    scroll_offset: usize,
    input_text: String,
    input_cursor_position: usize,
    waiting_for_ctrl_c: bool,
}

impl UIState {
    fn new() -> Self {
        Self {
            chat: ChatUIState::new(),
            scroll_offset: 0,
            input_text: String::new(),
            input_cursor_position: 0,
            waiting_for_ctrl_c: false,
        }
    }

    fn apply(&mut self, modification: ChatUIModification) -> anyhow::Result<()> {
        self.chat.apply(modification)?;

        // Auto-scroll to bottom when new messages are added
        let total_lines = self.calculate_total_lines();
        let visible_height = 20; // Default visible height, will be updated in render
        self.scroll_to_bottom(total_lines, visible_height);

        Ok(())
    }

    fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    fn scroll_down(&mut self, total_lines: usize, visible_height: usize) {
        if total_lines > visible_height && self.scroll_offset < total_lines - visible_height {
            self.scroll_offset += 1;
        }
    }

    fn scroll_to_bottom(&mut self, total_lines: usize, visible_height: usize) {
        if total_lines > visible_height {
            self.scroll_offset = total_lines - visible_height;
        } else {
            self.scroll_offset = 0;
        }
    }

    fn calculate_total_lines(&self) -> usize {
        let messages = self.chat.messages();
        let mut total_lines = 0;

        for message in messages {
            match message {
                ChatUIMessage::User(_) => {
                    total_lines += 1; // Each user message is one line
                }
                ChatUIMessage::System(s) => {
                    // Render markdown to count lines
                    let markdown_text = render_markdown_text(&s.text);
                    total_lines += markdown_text.lines.len();
                }
                ChatUIMessage::ToolCall(_) => {
                    total_lines += 1; // Each tool call is one line
                }
            }
        }

        total_lines
    }

    fn insert_char(&mut self, ch: char) {
        // Prevent newlines to keep input single-line
        if ch != '\n' {
            self.input_text.insert(self.input_cursor_position, ch);
            self.input_cursor_position += 1;
        }
    }

    fn delete_char_backward(&mut self) {
        if self.input_cursor_position > 0 {
            self.input_cursor_position -= 1;
            self.input_text.remove(self.input_cursor_position);
        }
    }

    fn delete_char_forward(&mut self) {
        if self.input_cursor_position < self.input_text.len() {
            self.input_text.remove(self.input_cursor_position);
        }
    }

    fn move_cursor_left(&mut self) {
        if self.input_cursor_position > 0 {
            self.input_cursor_position -= 1;
        }
    }

    fn move_cursor_right(&mut self) {
        if self.input_cursor_position < self.input_text.len() {
            self.input_cursor_position += 1;
        }
    }

    fn move_cursor_to_start(&mut self) {
        self.input_cursor_position = 0;
    }

    fn move_cursor_to_end(&mut self) {
        self.input_cursor_position = self.input_text.len();
    }

    fn clear_input(&mut self) {
        self.input_text.clear();
        self.input_cursor_position = 0;
    }

    fn submit_input(&mut self) -> String {
        let message = self.input_text.clone();
        self.clear_input();
        message
    }

    fn calculate_visible_height(&self, terminal_height: u16) -> u16 {
        let input_height = 3; // Fixed single-line input height
        terminal_height.saturating_sub(input_height).saturating_sub(2)
    }
}

pub async fn ui_loop(
    mut terminal: ratatui::Terminal<CrosstermBackend<Stdout>>,
    mut ui_rx: mpsc::UnboundedReceiver<ChatUIModification>,
    control_tx: mpsc::UnboundedSender<ControlMessage>,
    prompt: Option<String>,
) -> anyhow::Result<()> {
    let mut ui_state = UIState::new();

    // Send initial prompt if provided
    if let Some(prompt) = prompt {
        control_tx.send(ControlMessage::UserMessage(prompt))?;
    }
    let mut reader = EventStream::new();
    terminal.draw(|frame| {
        frame.render_widget(&ui_state, frame.area());
    })?;
    loop {
        let mut needs_redraw = false;

        tokio::select! {
            chat_modification = ui_rx.recv(), if !ui_rx.is_closed() => {
                let Some(chat_modification) = chat_modification else {
                    continue;
                };
                ui_state.apply(chat_modification)?;
                needs_redraw = true;
            },
            reader_event = reader.next() => {
                let Some(event) = reader_event else {
                    continue;
                };
                let event = event?;
                tracing::info!("Event: {event:?}");

                // Handle resize events
                if let Event::Resize(_, _) = event {
                    needs_redraw = true;
                }

                if let Event::Key(key) = event && key.kind == KeyEventKind::Press {
                    match key.code {
                        // Input handling
                        KeyCode::Char(ch) => {
                            // Check for quit sequence first
                            if ui_state.waiting_for_ctrl_c && ch == 'c' && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                                break; // Quit the application
                            }

                            // Check if this is Ctrl-X (start quit sequence)
                            if ch == 'x' && key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                                ui_state.waiting_for_ctrl_c = true;
                                needs_redraw = true;
                            } else {
                                // Reset quit sequence if any other key is pressed
                                if ui_state.waiting_for_ctrl_c {
                                    ui_state.waiting_for_ctrl_c = false;
                                }

                                // Check if this is a scroll command
                                match (ch, key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL), key.modifiers.contains(crossterm::event::KeyModifiers::ALT)) {
                                    ('p', true, false) => {
                                        ui_state.scroll_up();
                                    }
                                ('n', true, false) => {
                                    let total_lines = ui_state.calculate_total_lines();
                                    let terminal_height = terminal.size()?.height;
                                    let visible_height = ui_state.calculate_visible_height(terminal_height);
                                    ui_state.scroll_down(total_lines, visible_height as usize);
                                }
                                ('v', true, false) => {
                                    let total_lines = ui_state.calculate_total_lines();
                                    let terminal_height = terminal.size()?.height;
                                    let visible_height = ui_state.calculate_visible_height(terminal_height);
                                    for _ in 0..visible_height {
                                        ui_state.scroll_down(total_lines, visible_height as usize);
                                    }
                                }
                                ('v', false, true) => {
                                    let terminal_height = terminal.size()?.height;
                                    let visible_height = ui_state.calculate_visible_height(terminal_height);
                                    for _ in 0..visible_height {
                                        ui_state.scroll_up();
                                    }
                                }
                                    _ => {
                                        // Regular character input
                                        ui_state.insert_char(ch);
                                    }
                                }
                                needs_redraw = true;
                            }
                        }
                        KeyCode::Enter => {
                            // Enter: submit message
                            let message = ui_state.submit_input();
                            if !message.trim().is_empty() {
                                control_tx.send(ControlMessage::UserMessage(message))?;
                            }
                            needs_redraw = true;
                        }
                        KeyCode::Backspace => {
                            ui_state.delete_char_backward();
                            needs_redraw = true;
                        }
                        KeyCode::Delete => {
                            ui_state.delete_char_forward();
                            needs_redraw = true;
                        }
                        KeyCode::Left => {
                            ui_state.move_cursor_left();
                            needs_redraw = true;
                        }
                        KeyCode::Right => {
                            ui_state.move_cursor_right();
                            needs_redraw = true;
                        }
                        KeyCode::Home => {
                            ui_state.move_cursor_to_start();
                            needs_redraw = true;
                        }
                        KeyCode::End => {
                            ui_state.move_cursor_to_end();
                            needs_redraw = true;
                        }
                        _ => {}
                    }
                }
            }
        }

        // Single redraw at the end of each loop iteration
        if needs_redraw {
            terminal.draw(|frame| {
                frame.render_widget(&ui_state, frame.area());
            })?;
        }
    }
    tracing::info!("UI loop ended");
    anyhow::Ok(())
}

impl Widget for &UIState {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title = Line::from("Agent").bold();

        // Fixed single-line input height
        let input_height = 3; // 1 line for input + 2 for border

        let chat_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height.saturating_sub(input_height),
        };
        let input_area = Rect {
            x: area.x,
            y: area.y + chat_area.height,
            width: area.width,
            height: input_height,
        };

        // Render chat area
        let chat_block = Block::bordered().title(title.centered()).border_set(border::THICK);

        let messages = self.chat.messages();
        if messages.is_empty() {
            let empty_paragraph = Paragraph::new("No messages yet".dark_gray())
                .block(chat_block)
                .wrap(Wrap { trim: true });
            empty_paragraph.render(chat_area, buf);
        } else {
            // Build all messages into a single text with styled lines
            let mut lines = vec![];

            for message in messages {
                match message {
                    ChatUIMessage::User(u) => {
                        lines.push(Line::from(vec!["user: ".cyan().bold(), u.text.clone().into()]));
                    }
                    ChatUIMessage::System(s) => {
                        // Render the system message text as markdown
                        let markdown_text = render_markdown_text(&s.text);

                        // Add the "assistant: " prefix to the first line
                        if let Some(first_line) = markdown_text.lines.first() {
                            let mut prefixed_line = Line::from(vec!["assistant: ".yellow().bold()]);
                            prefixed_line.spans.extend(first_line.spans.clone());
                            lines.push(prefixed_line);
                        }

                        // Add the remaining lines from the markdown
                        for line in markdown_text.lines.iter().skip(1) {
                            lines.push(line.clone());
                        }
                    }
                    ChatUIMessage::ToolCall(tc) => match tc {
                        ChatUIToolCall::Generating { name, args } => {
                            lines.push(Line::from(vec![
                                "tool: ".magenta().bold(),
                                format!("{}({}) …", name, args).magenta(),
                            ]));
                        }
                        ChatUIToolCall::Executing { name, args } => {
                            lines.push(Line::from(vec![
                                "tool: ".magenta().bold(),
                                format!("{}({}) ", name, args).into(),
                                "running".magenta().bold(),
                            ]));
                        }
                        ChatUIToolCall::Complete { name, args, result } => {
                            let status = match result {
                                Ok(_) => "ok".green().bold(),
                                Err(_) => "error".red().bold(),
                            };
                            lines.push(Line::from(vec![
                                "tool: ".magenta().bold(),
                                format!("{}({}) ", name, args).into(),
                                status,
                            ]));
                        }
                    },
                }
            }

            // Apply scrolling - show only the visible portion
            let visible_height = chat_area.height.saturating_sub(2); // Account for border
            let total_lines = lines.len();

            // Calculate which lines to show based on scroll offset
            let start_line = self
                .scroll_offset
                .min(total_lines.saturating_sub(visible_height as usize));
            let end_line = (start_line + visible_height as usize).min(total_lines);

            let visible_lines = if start_line < total_lines {
                lines[start_line..end_line].to_vec()
            } else {
                vec![]
            };

            let paragraph = Paragraph::new(Text::from(visible_lines))
                .block(chat_block)
                .wrap(Wrap { trim: false });

            paragraph.render(chat_area, buf);

            // Add generating status in bottom left (only when generating)
            if let GeneratingState::Generating = self.chat.generating_state() {
                let status_text = "Generating...";
                let status_color = ratatui::style::Color::Yellow;

                let status_area = Rect {
                    x: chat_area.x + 1,
                    y: chat_area.y + chat_area.height - 1,
                    width: status_text.len() as u16 + 2,
                    height: 1,
                };

                let status_paragraph = Paragraph::new(status_text.fg(status_color)).wrap(Wrap { trim: true });
                status_paragraph.render(status_area, buf);
            }

            // Add performance stats and line indicator in bottom right
            let mut right_elements = vec![];

            // Add performance stats if available
            if let Some(stats) = self.chat.performance_stats() {
                let ttft_ms = stats.ttft.as_millis();
                let bytes_per_sec = stats.bytes_per_sec as u64;
                let human_bytes = format_size(bytes_per_sec, DECIMAL);
                let perf_text = format!(" TTFT: {}ms | {}/s", ttft_ms, human_bytes);
                right_elements.push(perf_text);
            }

            // Add line indicator if there's overflow
            if total_lines > visible_height as usize {
                let start_line = self.scroll_offset + 1; // Convert to 1-based indexing
                let end_line = (self.scroll_offset + visible_height as usize).min(total_lines);
                let indicator_text = format!("(lines {}-{} of {})", start_line, end_line, total_lines);
                right_elements.push(indicator_text);
            }

            // Render all right elements
            if !right_elements.is_empty() {
                let combined_text = right_elements.join(" | ");
                let indicator_area = Rect {
                    x: chat_area.x + chat_area.width.saturating_sub(combined_text.len() as u16 + 2),
                    y: chat_area.y + chat_area.height - 1,
                    width: (combined_text.len() as u16 + 2).min(chat_area.width),
                    height: 1,
                };

                let indicator_paragraph = Paragraph::new(combined_text.dark_gray()).wrap(Wrap { trim: true });
                indicator_paragraph.render(indicator_area, buf);
            }
        }

        // Render input area
        let input_block = Block::bordered().title("Input").border_set(border::THICK);

        // Create input text with cursor
        let mut input_text = self.input_text.clone();
        if self.input_cursor_position <= input_text.len() {
            input_text.insert(self.input_cursor_position, '|');
        }

        let input_paragraph = Paragraph::new(input_text).block(input_block).wrap(Wrap { trim: true });

        input_paragraph.render(input_area, buf);
    }
}
