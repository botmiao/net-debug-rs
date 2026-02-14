use std::time::Instant;

use anyhow::{Ok, Result};
use crossterm::event::{KeyCode, KeyModifiers};
use tokio::sync::mpsc::{channel, unbounded_channel, Receiver};

use crate::cli::args::{AppMode, Args, ProtocolType};
use crate::protocols::{common, Message, ProtocolHandler};
use crate::ui::layout::{AppLayout, LayoutType};
use crate::ui::widgets::{input_dialog::InputDialog, message_view::MessageView, status_bar::StatusBar};
// use crate

/// 应用程序状态
pub enum InputMode {
    Normal,
    Editing,
}

/// 数据显示格式
pub enum DisplayFormat {
    String,
    Hex,
}

/// 应用程序统计数据
pub struct Stats {
    pub sent_bytes: usize,
    pub received_bytes: usize,
    pub connected: bool,
    pub last_activity: Instant,
}

impl Default for Stats {
    fn default() -> Self {
        Self {
            sent_bytes: 0,
            received_bytes: 0,
            connected: false,
            last_activity: Instant::now(),
        }
    }
}

/// 主应用状态
pub struct App {
    /// 应用退出标志
    pub should_quit: bool,
    /// 输入模式
    input_mode: InputMode,
    /// 布局
    pub layout: AppLayout,
    /// 发送区状态
    pub send_view: MessageView,
    /// 接收区状态
    pub receive_view: MessageView,
    /// 状态栏
    pub status_bar: StatusBar,
    /// 输入对话框
    pub input_dialog: Option<InputDialog>,
    /// 统计数据
    pub stats: Stats,
    /// UI到服务端的消息发送通道
    // pub ui_to_server_tx: Option<Sender<Message>>,
    /// 协议处理器
    pub protocol_handler: Box<dyn ProtocolHandler + Send + Sync>,
    /// 服务端到UI的消息接收通道
    pub server_to_ui_rx: Option<Receiver<Message>>,
    pub args: Args,
}

impl App {
    pub async fn new(args: Args) -> Result<Self> {
        // 根据参数确定布局方式
        let layout_type = if args.vertical_layout {
            LayoutType::VerticalSplit
        } else {
            LayoutType::HorizontalSplit
        };

        let (server_to_ui_tx, server_to_ui_rx) = channel::<Message>(1000);
        // let mut ui_to_server_tx = None;

        // 设置发送和接收视图的标题
        let (send_title, recv_title) = match args.protocol {
            ProtocolType::Tcp => match args.mode {
                AppMode::Server => {
                    // TCP Server 模式
                    // let mut handler = common::create_protocol_handler("tcp", true, args.local_addr, None).await?;
                    // ui_to_server_tx = handler.get_ui_to_server_sender();

                    ("TCP Server Send", "TCP Server Receive")
                }
                AppMode::Client => ("TCP Client Send", "TCP Client Receive"),
            },
            ProtocolType::Udp => match args.mode {
                AppMode::Server => ("UDP Server Send", "UDP Server Receive"),
                AppMode::Client => ("UDP Client Send", "UDP Client Receive"),
            },
            ProtocolType::WebSocket => match args.mode {
                AppMode::Server => ("WebSocket Server Send", "WebSocket Server Receive"),
                AppMode::Client => ("WebSocket Client Send", "WebSocket Client Receive"),
            },
            ProtocolType::Http => match args.mode {
                AppMode::Server => ("HTTP Server Send", "HTTP Server Receive"),
                AppMode::Client => ("HTTP Client Send", "HTTP Client Receive"),
            },
            ProtocolType::Http2 => match args.mode {
                AppMode::Server => ("HTTP/2 Server Send", "HTTP/2 Server Receive"),
                AppMode::Client => ("HTTP/2 Client Send", "HTTP/2 Client Receive"),
            },
            ProtocolType::Http3 => match args.mode {
                AppMode::Server => ("HTTP/3 Server Send", "HTTP/3 Server Receive"),
                AppMode::Client => ("HTTP/3 Client Send", "HTTP/3 Client Receive"),
            },
        };

        let handler =
            common::create_protocol_handler("tcp", true, Some(server_to_ui_tx), args.local_addr, None).await?;

        let app = Self {
            should_quit: false,
            input_mode: InputMode::Normal,
            layout: AppLayout::new(layout_type),
            send_view: MessageView::new(send_title),
            receive_view: MessageView::new(recv_title),
            status_bar: StatusBar::default(),
            input_dialog: None,
            stats: Stats::default(),
            // ui_to_server_tx,
            protocol_handler: handler,
            server_to_ui_rx: Some(server_to_ui_rx),
            args,
        };

        Ok(app)
    }

    pub fn receive_message(&mut self) {
        // 从 Option 中取出接收器的所有权
        if let Some(server_to_ui_rx) = self.server_to_ui_rx.as_mut() {
            // 处理接收到的消息
            match server_to_ui_rx.try_recv() {
                core::result::Result::Ok(message) => match message.content {
                    common::MessageType::Text(txt) => {
                        self.add_received_message(txt, None);
                    }
                    common::MessageType::ClientConnected => {
                        self.receive_view
                            .add_connection(&message.connection_info.unwrap().connection_id);
                    }
                    common::MessageType::ClientDisconnected => {
                        self.receive_view
                            .close_connection_by_title(&message.connection_info.unwrap().connection_id);
                    }
                    common::MessageType::Binary(data) => {
                        // 将二进制数据显示为十六进制
                        let hex_str: String = data.iter().map(|b| format!("{:02x}", b)).collect();
                        self.add_received_message(format!("[Binary] {}", hex_str), None);
                    }
                    common::MessageType::Hex(hex_str) => {
                        // 十六进制消息直接显示
                        self.add_received_message(format!("[Hex] {}", hex_str), None);
                    }
                },
                core::result::Result::Err(_) => {
                    // 没有消息可接收，继续执行
                }
            }
        }
    }

    /// 处理按键事件
    pub fn handle_key_event(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_mode_key(key, modifiers),
            InputMode::Editing => self.handle_editing_mode_key(key, modifiers),
        }
    }

    /// 处理正常模式键盘输入
    fn handle_normal_mode_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match (key, modifiers) {
            // 退出应用
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // 输入模式 (I)
            (KeyCode::Char('i'), KeyModifiers::NONE) => {
                self.input_mode = InputMode::Editing;
                self.input_dialog = Some(InputDialog::new());
            }
            _ => {}
        }
        Ok(())
    }

    /// 处理编辑模式键盘输入
    fn handle_editing_mode_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        if let Some(dialog) = &mut self.input_dialog {
            match key {
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.input_dialog = None;
                }
                KeyCode::Enter => {
                    // 获取输入内容并发送
                    if let Some(input) = dialog.submit() {
                        // 处理输入的内容，实际发送逻辑将由具体协议实现
                        self.send_message(input);
                    }
                    self.input_mode = InputMode::Normal;
                    self.input_dialog = None;
                }
                KeyCode::Char(c) => {
                    dialog.input.push(c);
                }
                KeyCode::Backspace => {
                    dialog.input.pop();
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// 发送消息（异步版本，用于在异步上下文中调用）
    pub async fn send_message_async(&mut self, message: String) {
        // 更新统计数据
        self.stats.sent_bytes += message.len();
        self.stats.last_activity = Instant::now();

        // 添加消息到发送视图
        self.send_view
            .add_message(format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), message.clone()));

        // 通过协议处理器发送消息
        let message_type = common::MessageType::Text(message);
        let target = None; // 可以扩展为发送到特定客户端
        
        // 直接调用异步方法
        let _ = self.protocol_handler.send_message(message_type, target).await;
    }

    fn send_message(&mut self, message: String) {
        // 创建一个本地任务来执行异步发送
        // 注意：这里我们不在同步方法中等待结果，而是让消息在后台发送
        let message_type = common::MessageType::Text(message.clone());
        let target = None;
        
        // 更新统计数据和 UI
        self.stats.sent_bytes += message.len();
        self.stats.last_activity = Instant::now();
        self.send_view
            .add_message(format!("[{}] {}", chrono::Local::now().format("%H:%M:%S"), message));
        
        // 尝试获取发送器并发送消息
        // 注意：由于不能直接在同步方法中调用 async 方法，
        // 我们在这里只是记录消息，实际的发送应该通过其他机制处理
        // 或者使用 spawn 来在后台执行
        if let Some(tx) = self.protocol_handler.get_ui_to_server_sender() {
            let msg = common::Message {
                content: message_type,
                direction: common::MessageDirection::Sent,
                timestamp: chrono::Local::now(),
                connection_info: None,
            };
            // 使用 tokio::spawn 在后台发送，不阻塞当前线程
            tokio::spawn(async move {
                let _ = tx.send(msg).await;
            });
        }
    }

    /// 添加接收到的消息
    pub fn add_received_message(&mut self, message: String, from: Option<String>) {
        // 更新统计数据
        self.stats.received_bytes += message.len();
        self.stats.last_activity = Instant::now();

        // 添加消息到接收视图
        let prefix = if let Some(addr) = from {
            format!("[{}] [{}]", chrono::Local::now().format("%H:%M:%S"), addr)
        } else {
            format!("[{}]", chrono::Local::now().format("%H:%M:%S"))
        };

        self.receive_view.add_message(format!("{} {}", prefix, message));
    }

    /// 更新连接状态
    pub fn set_connected(&mut self, connected: bool) {
        self.stats.connected = connected;
    }
}
