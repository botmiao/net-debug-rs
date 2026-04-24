use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem},
    Frame,
};

use crate::ui::widgets::tabs::TabsState;

/// 消息视图组件
pub struct MessageView {
    /// 标题
    title: String,
    /// 消息列表
    messages: Vec<String>,
    /// 是否有多个连接 (需要使用 tabs)
    has_multiple_connections: bool,
    /// 标签页状态 (用于多连接)
    tabs: Option<TabsState>,
    /// 滚动位置
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
    /// 添加消息
    pub fn add_message(&mut self, message: String) {
        self.messages.push(message);

        // 自动滚动到底部
        if self.messages.len() > 100 {
            // 保持最新的100条消息，避免内存占用过多
            self.messages = self.messages.split_off(self.messages.len() - 100);
        }
    }

    /// 添加消息到指定标签页
    pub fn add_message_to_tab(&mut self, tab_index: usize, message: String) {
        if let Some(tabs) = &mut self.tabs {
            tabs.add_message(tab_index, message);
        } else {
            // 如果没有 tabs，创建一个
            self.initialize_tabs();
            if let Some(tabs) = &mut self.tabs {
                tabs.add_message(tab_index, message);
            }
        }
    }

    /// 初始化标签页
    pub fn initialize_tabs(&mut self) {
        if self.tabs.is_none() {
            self.tabs = Some(TabsState::new(vec!["Default".to_string()]));
            self.has_multiple_connections = true;

            // 将现有消息移到默认标签页
            if let Some(tabs) = &mut self.tabs {
                for msg in &self.messages {
                    tabs.add_message(0, msg.clone());
                }
            }
        }
    }

    /// 添加新的连接标签页
    pub fn add_connection(&mut self, connection_name: &str) {
        self.has_multiple_connections = true;
        if let Some(tabs) = &mut self.tabs {
            tabs.add_tab(connection_name.to_string());
        } else {
            let mut tabs = TabsState::new(vec!["Default".to_string()]);
            tabs.add_tab(connection_name.to_string());
            self.tabs = Some(tabs);
        }
    }

    /// 关闭连接标签页
    pub fn close_connection(&mut self, tab_index: usize) {
        if let Some(tabs) = &mut self.tabs {
            tabs.remove_tab(tab_index);
            if tabs.titles.len() <= 1 {
                self.has_multiple_connections = false;
            }
        }
    }

    pub fn close_connection_by_title(&mut self, title: &str) {
        if let Some(tabs) = &mut self.tabs {
            tabs.remove_tab_by_title(title);
            if tabs.titles.len() <= 1 {
                self.has_multiple_connections = false;
            }
        }
    }

    /// 清除所有消息
    pub fn clear(&mut self) {
        self.messages.clear();
        self.scroll = 0;

        if let Some(tabs) = &mut self.tabs {
            for content in &mut tabs.contents {
                content.clear();
            }
        }
    }

    /// 向上滚动
    pub fn scroll_up(&mut self) {
        if self.scroll > 0 {
            self.scroll -= 1;
        }
    }

    /// 向下滚动
    pub fn scroll_down(&mut self, max_visible: usize) {
        if self.scroll + max_visible < self.messages.len() {
            self.scroll += 1;
        }
    }

    /// 滚动到顶部
    pub fn scroll_to_top(&mut self) {
        self.scroll = 0;
    }

    /// 滚动到底部
    pub fn scroll_to_bottom(&mut self, max_visible: usize) {
        if self.messages.len() > max_visible {
            self.scroll = self.messages.len() - max_visible;
        } else {
            self.scroll = 0;
        }
    }

    /// 下一个标签页
    pub fn next_tab(&mut self) {
        if let Some(tabs) = &mut self.tabs {
            tabs.next();
        }
    }

    /// 上一个标签页
    pub fn prev_tab(&mut self) {
        if let Some(tabs) = &mut self.tabs {
            tabs.previous();
        }
    }
    /// 绘制视图
    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        // 创建一个带边框的块
        let block = Block::default().title(self.title.clone()).borders(Borders::ALL);

        // 绘制边框
        frame.render_widget(block.clone(), area);

        // 计算内部区域
        let inner_area = block.inner(area);

        // 如果有多个连接，使用标签页布局
        if self.has_multiple_connections && self.tabs.is_some() {
            // 在此区域渲染标签页和内容
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3), // 标签栏高度
                    Constraint::Min(0),    // 消息内容区高度
                ])
                .split(inner_area);

            // 绘制标签页
            if let Some(tabs) = &self.tabs {
                // 渲染标签页标题
                let titles: Vec<Line> = tabs.titles.iter().map(|t| Line::from(t.as_str())).collect();

                let tabs_widget = ratatui::widgets::Tabs::new(titles)
                    .block(Block::default().borders(Borders::BOTTOM))
                    .select(tabs.index)
                    .style(Style::default())
                    .highlight_style(Style::default().fg(Color::LightCyan));

                frame.render_widget(tabs_widget, chunks[0]);

                // 渲染当前选中标签页的内容
                if tabs.index < tabs.contents.len() {
                    let messages = &tabs.contents[tabs.index];
                    let max_visible = chunks[1].height as usize;

                    let items: Vec<ListItem> = if !messages.is_empty() {
                        let start_idx = if messages.len() > max_visible {
                            messages.len() - max_visible + self.scroll
                        } else {
                            0
                        };

                        let visible_messages = &messages[start_idx.min(messages.len())..];

                        visible_messages
                            .iter()
                            .map(|m| ListItem::new(Line::from(vec![Span::raw(m)])))
                            .collect()
                    } else {
                        Vec::new()
                    };

                    // 创建列表小部件
                    let list = List::new(items)
                        .style(Style::default())
                        .highlight_style(Style::default().fg(Color::LightCyan));

                    frame.render_widget(list, chunks[1]);
                }
            }
        } else {
            // 无标签页，直接显示消息
            let max_visible = inner_area.height as usize;

            // 创建消息列表
            let start_idx = if self.messages.len() > max_visible {
                self.messages.len() - max_visible + self.scroll
            } else {
                0
            };

            let visible_messages = &self.messages[start_idx.min(self.messages.len())..];

            let items: Vec<ListItem> = visible_messages
                .iter()
                .map(|m| {
                    // 每条消息作为列表项
                    ListItem::new(Line::from(vec![Span::raw(m)]))
                })
                .collect();

            // 创建列表小部件
            let list = List::new(items)
                .style(Style::default())
                .highlight_style(Style::default().fg(Color::LightCyan));

            frame.render_widget(list, inner_area);
        }
    }
}
