use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Line,
    widgets::{Block, Borders, Clear, Paragraph, Tabs},
    Frame,
};

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

/// 数据发送格式
pub enum FormatType {
    String,
    Hex,
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

    /// 添加客户端
    pub fn add_client(&mut self, client: String) {
        self.clients.push(client);
        if self.selected_client.is_none() && !self.clients.is_empty() {
            self.selected_client = Some(0);
        }
    }

    /// 切换格式类型
    pub fn toggle_format(&mut self) {
        self.format_type = match self.format_type {
            FormatType::String => FormatType::Hex,
            FormatType::Hex => FormatType::String,
        };
    }

    /// 提交输入并返回内容
    pub fn submit(&self) -> Option<String> {
        if self.input.is_empty() {
            None
        } else {
            Some(self.input.clone())
        }
    }

    /// 绘制对话框
    pub fn draw(&self, frame: &mut Frame) {
        // 计算对话框的尺寸和位置
        let area = frame.area();
        let width = area.width.min(60);
        let height = 10;
        let x = (area.width - width) / 2;
        let y = (area.height - height) / 2;
        let dialog_area = Rect::new(x, y, width, height);

        // 创建清除层，防止对话框下方内容显示
        frame.render_widget(Clear, dialog_area);

        // 创建对话框边框
        let block = Block::default()
            .title("Send Message")
            .borders(Borders::ALL)
            .style(Style::default().bg(Color::DarkGray));

        // 创建垂直布局
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .margin(1)
            .constraints([
                Constraint::Length(1),  // 格式选择
                Constraint::Length(1),  // 客户端选择
                Constraint::Length(1),  // 间距
                Constraint::Min(3),     // 输入区域
            ])
            .split(dialog_area);

        // 绘制对话框边框
        frame.render_widget(block, dialog_area);

        // 绘制格式选择标签 — 将标签和 Tabs 并排放置
        let format_chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Length(8), Constraint::Min(0)])
            .split(chunks[0]);

        let format_tabs = Tabs::new(vec![
            Line::from("String"),
            Line::from("Hex"),
        ])
        .select(match self.format_type {
            FormatType::String => 0,
            FormatType::Hex => 1,
        })
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().fg(Color::Yellow));
        frame.render_widget(Paragraph::new("Format:"), format_chunks[0]);
        frame.render_widget(format_tabs, format_chunks[1]);

        // 如果有客户端，绘制客户端选择 — 将标签和 Tabs 并排放置
        if !self.clients.is_empty() {
            let client_chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Length(8), Constraint::Min(0)])
                .split(chunks[1]);

            let client_names: Vec<Line> = self.clients.iter().map(|c| Line::from(c.clone())).collect();
            let client_tabs = Tabs::new(client_names)
                .select(self.selected_client.unwrap_or(0))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow));

            frame.render_widget(Paragraph::new("Client:"), client_chunks[0]);
            frame.render_widget(client_tabs, client_chunks[1]);
        }

        // 绘制输入区域
        let input_block = Block::default()
            .borders(Borders::ALL)
            .style(Style::default());

        let input_paragraph = Paragraph::new(self.input.as_str())
            .block(input_block)
            .style(Style::default().fg(Color::White));

        frame.render_widget(input_paragraph, chunks[3]);

        // 显示光标
        frame.set_cursor_position((
            chunks[3].x + 1 + self.input.len() as u16,
            chunks[3].y + 1,
        ));
    }
}