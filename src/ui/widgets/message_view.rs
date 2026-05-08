use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Tabs},
    Frame,
};

use crate::ui::widgets::tabs::TabsState;

const MAX_MESSAGES: usize = 100;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DataDisplayMode {
    String,
    Hex,
}

impl DataDisplayMode {
    pub fn label(self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Hex => "Hex",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum MessageBody {
    Text(String),
    Bytes(Vec<u8>),
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct MessageEntry {
    header: String,
    body: MessageBody,
}

/// 消息视图组件
pub struct MessageView {
    title: String,
    /// 预格式化的消息列表（非 tab 模式）
    messages: Vec<MessageEntry>,
    has_multiple_connections: bool,
    tabs: Option<TabsState<MessageEntry>>,
    display_mode: DataDisplayMode,
    scroll: usize,
}

impl MessageView {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            messages: Vec::new(),
            has_multiple_connections: false,
            tabs: None,
            display_mode: DataDisplayMode::String,
            scroll: 0,
        }
    }

    /// 添加消息（两行格式：header 和 content）
    pub fn add_message(&mut self, header: String, content: String, tab: Option<&str>) {
        self.add_entry(
            MessageEntry {
                header,
                body: MessageBody::Text(content),
            },
            tab,
        );
    }

    /// 添加原始字节消息，显示时再按当前模式转换为 String 或 Hex。
    pub fn add_bytes_message(&mut self, header: String, content: Vec<u8>, tab: Option<&str>) {
        self.add_entry(
            MessageEntry {
                header,
                body: MessageBody::Bytes(content),
            },
            tab,
        );
    }

    fn add_entry(&mut self, entry: MessageEntry, tab: Option<&str>) {
        if let Some(tab_name) = tab {
            self.ensure_tabs();
            if let Some(tabs) = &mut self.tabs {
                push_with_trim(&mut tabs.contents[0], entry.clone());
                if let Some(idx) = tabs.titles.iter().position(|t| t == tab_name) {
                    if idx != 0 {
                        push_with_trim(&mut tabs.contents[idx], entry);
                    }
                }
            }
        } else if self.has_multiple_connections {
            if let Some(tabs) = &mut self.tabs {
                push_with_trim(&mut tabs.contents[tabs.index], entry);
            }
        } else {
            push_with_trim(&mut self.messages, entry);
        }
    }

    fn ensure_tabs(&mut self) {
        if self.tabs.is_none() {
            let mut tabs = TabsState::new(vec!["All".to_string()]);
            for msg in self.messages.drain(..) {
                tabs.contents[0].push(msg);
            }
            self.tabs = Some(tabs);
            self.has_multiple_connections = true;
        }
    }

    pub fn add_connection(&mut self, connection_name: &str) {
        self.ensure_tabs();
        if let Some(tabs) = &mut self.tabs {
            tabs.add_tab(connection_name.to_string());
        }
    }

    pub fn close_connection_by_title(&mut self, title: &str) {
        if let Some(tabs) = &mut self.tabs {
            tabs.remove_tab_by_title(title);
            if tabs.titles.len() <= 1 {
                self.has_multiple_connections = false;
                self.messages = tabs.contents.first().cloned().unwrap_or_default();
                self.tabs = None;
            }
        }
    }

    pub fn next_tab(&mut self) {
        if let Some(tabs) = &mut self.tabs {
            tabs.next();
        }
    }

    pub fn prev_tab(&mut self) {
        if let Some(tabs) = &mut self.tabs {
            tabs.previous();
        }
    }

    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll = self.scroll.saturating_add(lines);
    }

    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll = self.scroll.saturating_sub(lines);
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll = 0;
    }

    pub fn toggle_display_mode(&mut self) {
        self.display_mode = match self.display_mode {
            DataDisplayMode::String => DataDisplayMode::Hex,
            DataDisplayMode::Hex => DataDisplayMode::String,
        };
    }

    pub fn set_display_mode(&mut self, mode: DataDisplayMode) {
        self.display_mode = mode;
    }

    pub fn display_mode(&self) -> DataDisplayMode {
        self.display_mode
    }

    #[cfg(test)]
    fn scroll_offset(&self) -> usize {
        self.scroll
    }

    #[cfg(test)]
    fn formatted_messages_for_test(&self) -> Vec<String> {
        self.messages
            .iter()
            .map(|entry| format_entry(entry, self.display_mode))
            .collect()
    }

    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .title(self.title.as_str())
            .borders(Borders::ALL);
        frame.render_widget(block.clone(), area);
        let inner_area = block.inner(area);

        if self.has_multiple_connections && self.tabs.is_some() {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Length(3), Constraint::Min(0)])
                .split(inner_area);

            if let Some(tabs) = &self.tabs {
                let titles: Vec<Line> =
                    tabs.titles.iter().map(|t| Line::from(t.as_str())).collect();
                let tabs_widget = Tabs::new(titles)
                    .block(Block::default().borders(Borders::BOTTOM))
                    .select(tabs.index)
                    .highlight_style(Style::default().fg(Color::LightCyan));
                frame.render_widget(tabs_widget, chunks[0]);

                if tabs.index < tabs.contents.len() {
                    render_messages(
                        frame,
                        chunks[1],
                        &tabs.contents[tabs.index],
                        self.display_mode,
                        self.scroll,
                    );
                }
            }
        } else {
            render_messages(
                frame,
                inner_area,
                &self.messages,
                self.display_mode,
                self.scroll,
            );
        }
    }
}

fn format_entry(entry: &MessageEntry, mode: DataDisplayMode) -> String {
    let content = match (&entry.body, mode) {
        (MessageBody::Text(text), DataDisplayMode::String) => text.clone(),
        (MessageBody::Text(text), DataDisplayMode::Hex) => {
            crate::utils::data_format::bytes_to_hex(text.as_bytes())
        }
        (MessageBody::Bytes(bytes), DataDisplayMode::String) => {
            crate::utils::data_format::bytes_to_string(bytes)
        }
        (MessageBody::Bytes(bytes), DataDisplayMode::Hex) => {
            crate::utils::data_format::bytes_to_hex(bytes)
        }
    };

    format!("── {} ──\n  {}", entry.header, content)
}

fn push_with_trim(list: &mut Vec<MessageEntry>, msg: MessageEntry) {
    list.push(msg);
    if list.len() > MAX_MESSAGES {
        list.drain(0..list.len() - MAX_MESSAGES);
    }
}

/// 渲染消息列表 — 只为可见区域构建 ListItem
fn render_messages(
    frame: &mut Frame,
    area: Rect,
    messages: &[MessageEntry],
    display_mode: DataDisplayMode,
    scroll: usize,
) {
    if messages.is_empty() {
        return;
    }

    let max_visible = area.height as usize;
    if max_visible == 0 {
        return;
    }

    // 计算每条消息的行数和总行数
    let heights: Vec<usize> = messages
        .iter()
        .map(|m| format_entry(m, display_mode).matches('\n').count() + 1)
        .collect();
    let total_lines: usize = heights.iter().sum();

    // 确定 skip 的行数
    let skip_lines = if total_lines > max_visible {
        (total_lines - max_visible).saturating_sub(scroll)
    } else {
        0
    };

    // 跳过前 skip_lines 行，找到起始消息索引和行偏移
    let mut acc = 0;
    let mut start_idx = 0;
    let mut line_offset = 0;
    for (i, &h) in heights.iter().enumerate() {
        if acc + h > skip_lines {
            start_idx = i;
            line_offset = skip_lines - acc;
            break;
        }
        acc += h;
    }

    // 构建可见区域的 ListItem
    let mut visible_items = Vec::new();
    let mut used_lines = 0;

    for entry in &messages[start_idx..] {
        if used_lines >= max_visible {
            break;
        }
        let msg = format_entry(entry, display_mode);
        let lines: Vec<Line> = msg
            .split('\n')
            .enumerate()
            .filter(|(i, _)| start_idx == 0 || *i >= line_offset || used_lines > 0)
            .map(|(_, l)| {
                if l.starts_with("──") {
                    Line::from(Span::styled(
                        l.to_string(),
                        Style::default().fg(Color::DarkGray),
                    ))
                } else {
                    Line::from(l.to_string())
                }
            })
            .collect();

        used_lines += lines.len();
        visible_items.push(ListItem::new(lines));
    }

    let list = List::new(visible_items);
    frame.render_widget(list, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_tab_contains_messages_added_to_connection_tabs() {
        let mut view = MessageView::new("Receive");

        view.add_connection("127.0.0.1:12345");
        view.add_message(
            "12:00:00 | 127.0.0.1:12345".to_string(),
            "hello".to_string(),
            Some("127.0.0.1:12345"),
        );

        let tabs = view.tabs.as_ref().unwrap();
        assert_eq!(tabs.contents[0].len(), 1);
        assert_eq!(tabs.contents[1].len(), 1);
        assert_eq!(tabs.contents[0][0], tabs.contents[1][0]);
    }

    #[test]
    fn scroll_up_and_down_changes_scroll_offset() {
        let mut view = MessageView::new("Receive");

        assert_eq!(view.scroll_offset(), 0);
        view.scroll_up(3);
        assert_eq!(view.scroll_offset(), 3);
        view.scroll_down(1);
        assert_eq!(view.scroll_offset(), 2);
        view.scroll_to_bottom();
        assert_eq!(view.scroll_offset(), 0);
    }

    #[test]
    fn hex_display_mode_formats_text_and_binary_payloads_as_hex() {
        let mut view = MessageView::new("Receive");

        view.set_display_mode(DataDisplayMode::Hex);
        view.add_message("12:00:00".to_string(), "中".to_string(), None);
        view.add_bytes_message("12:00:01".to_string(), vec![0xFF, 0x00], None);

        assert!(view.formatted_messages_for_test()[0].contains("E4 B8 AD"));
        assert!(view.formatted_messages_for_test()[1].contains("FF 00"));
    }
}
