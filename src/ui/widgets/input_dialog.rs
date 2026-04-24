use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
    Frame,
};

/// 数据发送格式
pub enum FormatType {
    String,
    Hex,
}

/// 提交结果
pub struct SubmitResult {
    pub input: String,
    pub format_hex: bool,
    pub target_client: Option<String>,
}

/// 输入对话框组件
pub struct InputDialog {
    /// 用户输入的文本
    pub input: String,
    /// 数据发送格式 (String/Hex)
    pub format_type: FormatType,
    /// 当前选择的客户端索引
    pub selected_client: Option<usize>,
    /// 可用的客户端列表
    pub clients: Vec<String>,
}

impl InputDialog {
    pub fn new() -> Self {
        Self {
            input: String::new(),
            format_type: FormatType::String,
            selected_client: None,
            clients: Vec::new(),
        }
    }

    pub fn add_client(&mut self, client: String) {
        self.clients.push(client);
        if self.selected_client.is_none() && !self.clients.is_empty() {
            self.selected_client = Some(0);
        }
    }

    pub fn toggle_format(&mut self) {
        self.format_type = match self.format_type {
            FormatType::String => FormatType::Hex,
            FormatType::Hex => FormatType::String,
        };
    }

    pub fn submit(&self) -> Option<SubmitResult> {
        if self.input.is_empty() {
            None
        } else {
            Some(SubmitResult {
                input: self.input.clone(),
                format_hex: matches!(self.format_type, FormatType::Hex),
                target_client: self.selected_client.map(|i| self.clients[i].clone()),
            })
        }
    }

    pub fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        if area.width == 0 || area.height == 0 {
            return;
        }

        let width = area.width.min(60);
        let height = area.height.min(10);
        let x = area.width.saturating_sub(width) / 2;
        let y = area.height.saturating_sub(height) / 2;
        let dialog_area = Rect::new(x, y, width, height);

        frame.render_widget(Clear, dialog_area);

        let block = Block::default()
            .title("Send Message")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Length(1),
                Constraint::Min(3),
            ])
            .split(dialog_area);

        frame.render_widget(block, dialog_area);

        // 格式选择: 标签 + Tabs 并排
        let format_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(8), Constraint::Min(0)])
            .split(chunks[0]);

        let format_tabs = Tabs::new(vec![Line::from("String"), Line::from("Hex")])
            .select(match self.format_type {
                FormatType::String => 0,
                FormatType::Hex => 1,
            })
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow));
        frame.render_widget(Paragraph::new("Format:"), format_chunks[0]);
        frame.render_widget(format_tabs, format_chunks[1]);

        // 客户端选择
        if !self.clients.is_empty() {
            let client_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(8), Constraint::Min(0)])
                .split(chunks[1]);

            let client_tabs = Tabs::new(
                self.clients
                    .iter()
                    .map(|c| Line::from(c.clone()))
                    .collect::<Vec<_>>(),
            )
            .select(self.selected_client.unwrap_or(0))
            .style(Style::default().fg(Color::White))
            .highlight_style(Style::default().fg(Color::Yellow));

            frame.render_widget(Paragraph::new("Client:"), client_chunks[0]);
            frame.render_widget(client_tabs, client_chunks[1]);
        }

        // 输入区域
        let format_hint = match self.format_type {
            FormatType::String => "",
            FormatType::Hex => " (hex mode)",
        };
        let input_block = Block::default()
            .title(format!("Input{}", format_hint))
            .borders(Borders::ALL);

        let input_paragraph = Paragraph::new(self.input.as_str())
            .block(input_block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(input_paragraph, chunks[3]);

        if chunks[3].width > 2 && chunks[3].height > 2 {
            let cursor_x = chunks[3]
                .x
                .saturating_add(1)
                .saturating_add(self.input.len() as u16)
                .min(chunks[3].right().saturating_sub(2));
            frame.set_cursor_position((cursor_x, chunks[3].y + 1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ratatui::{backend::TestBackend, Terminal};

    #[test]
    fn draw_does_not_panic_on_short_terminal_after_resize() {
        let backend = TestBackend::new(20, 5);
        let mut terminal = Terminal::new(backend).unwrap();
        let dialog = InputDialog::new();

        terminal.draw(|frame| dialog.draw(frame)).unwrap();
    }
}
