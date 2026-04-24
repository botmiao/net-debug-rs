use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Tabs},
    Frame,
};

use crate::ui::widgets::tabs::TabsState;

const MAX_MESSAGES: usize = 100;

/// 消息视图组件
pub struct MessageView {
    title: String,
    /// 预格式化的消息列表（非 tab 模式）
    messages: Vec<String>,
    has_multiple_connections: bool,
    tabs: Option<TabsState>,
    scroll: usize,
}

impl MessageView {
    pub fn new(title: &str) -> Self {
        Self {
            title: title.to_string(),
            messages: Vec::new(),
            has_multiple_connections: false,
            tabs: None,
            scroll: 0,
        }
    }

    /// 添加消息（两行格式：header 和 content）
    pub fn add_message(&mut self, header: String, content: String, tab: Option<&str>) {
        let formatted = format_message(&header, &content);

        if let Some(tab_name) = tab {
            self.ensure_tabs();
            if let Some(tabs) = &mut self.tabs {
                push_with_trim(&mut tabs.contents[0], formatted.clone());
                if let Some(idx) = tabs.titles.iter().position(|t| t == tab_name) {
                    if idx != 0 {
                        push_with_trim(&mut tabs.contents[idx], formatted);
                    }
                }
            }
        } else if self.has_multiple_connections {
            if let Some(tabs) = &mut self.tabs {
                push_with_trim(&mut tabs.contents[tabs.index], formatted);
            }
        } else {
            push_with_trim(&mut self.messages, formatted);
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
                    render_messages(frame, chunks[1], &tabs.contents[tabs.index], self.scroll);
                }
            }
        } else {
            render_messages(frame, inner_area, &self.messages, self.scroll);
        }
    }
}

fn format_message(header: &str, content: &str) -> String {
    format!("── {} ──\n  {}", header, content)
}

fn push_with_trim(list: &mut Vec<String>, msg: String) {
    list.push(msg);
    if list.len() > MAX_MESSAGES {
        list.drain(0..list.len() - MAX_MESSAGES);
    }
}

/// 渲染消息列表 — 只为可见区域构建 ListItem
fn render_messages(frame: &mut Frame, area: Rect, messages: &[String], scroll: usize) {
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
        .map(|m| m.matches('\n').count() + 1)
        .collect();
    let total_lines: usize = heights.iter().sum();

    // 确定 skip 的行数
    let skip_lines = if total_lines > max_visible {
        (total_lines - max_visible + scroll).min(total_lines - max_visible)
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

    for msg in &messages[start_idx..] {
        if used_lines >= max_visible {
            break;
        }
        let lines: Vec<Line> = msg
            .split('\n')
            .enumerate()
            .filter(|(i, _)| start_idx == 0 || *i >= line_offset || used_lines > 0)
            .map(|(_, l)| {
                if l.starts_with("──") {
                    Line::from(Span::styled(l, Style::default().fg(Color::DarkGray)))
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
}
