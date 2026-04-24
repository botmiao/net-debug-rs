use std::time::Instant;

use anyhow::Result;
use crossterm::event::{KeyCode, KeyModifiers};
use tokio::sync::mpsc::{channel, Receiver};

use crate::cli::args::{AppMode, Args, ProtocolType};
use crate::protocols::{common, Message, ProtocolHandler};
use crate::ui::layout::{AppLayout, LayoutType};
use crate::ui::widgets::{input_dialog::InputDialog, message_view::MessageView, status_bar::StatusBar};

/// 应用程序状态
pub enum InputMode {
    Normal,
    Editing,
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

/// 判断协议是否支持手动发送消息
fn protocol_supports_send(protocol: &ProtocolType) -> bool {
    !matches!(protocol, ProtocolType::Http | ProtocolType::Http2 | ProtocolType::Http3)
}

/// 主应用状态
pub struct App {
    pub should_quit: bool,
    input_mode: InputMode,
    pub layout: AppLayout,
    pub send_view: MessageView,
    pub receive_view: MessageView,
    pub status_bar: StatusBar,
    pub input_dialog: Option<InputDialog>,
    pub stats: Stats,
    pub protocol_handler: Box<dyn ProtocolHandler + Send + Sync>,
    pub server_to_ui_rx: Option<Receiver<Message>>,
    pub args: Args,
}

impl App {
    pub async fn new(args: Args) -> Result<Self> {
        let layout_type = if args.vertical_layout {
            LayoutType::VerticalSplit
        } else {
            LayoutType::HorizontalSplit
        };

        let (server_to_ui_tx, server_to_ui_rx) = channel::<Message>(1000);

        let (send_title, recv_title) = match args.protocol {
            ProtocolType::Tcp => match args.mode {
                AppMode::Server => ("TCP Server Send", "TCP Server Receive"),
                AppMode::Client => ("TCP Client Send", "TCP Client Receive"),
            },
            ProtocolType::Udp => match args.mode {
                AppMode::Server => ("UDP Server Send", "UDP Server Receive"),
                AppMode::Client => ("UDP Client Send", "UDP Client Receive"),
            },
            ProtocolType::WebSocket => match args.mode {
                AppMode::Server => ("WS Server Send", "WS Server Receive"),
                AppMode::Client => ("WS Client Send", "WS Client Receive"),
            },
            ProtocolType::Http => ("HTTP Server Send", "HTTP Server Receive"),
            ProtocolType::Http2 => ("HTTP/2 Server Send", "HTTP/2 Server Receive"),
            ProtocolType::Http3 => ("HTTP/3 Server Send", "HTTP/3 Server Receive"),
        };

        let handler = {
            let protocol_name = match args.protocol {
                ProtocolType::Tcp => "tcp",
                ProtocolType::Udp => "udp",
                ProtocolType::WebSocket => "websocket",
                ProtocolType::Http => "http",
                ProtocolType::Http2 => "http2",
                ProtocolType::Http3 => "http3",
            };
            let is_server = args.mode == AppMode::Server;
            common::create_protocol_handler(
                protocol_name,
                is_server,
                Some(server_to_ui_tx),
                args.local_addr,
                args.remote_addr,
            ).await?
        };

        let app = Self {
            should_quit: false,
            input_mode: InputMode::Normal,
            layout: AppLayout::new(layout_type),
            send_view: MessageView::new(send_title),
            receive_view: MessageView::new(recv_title),
            status_bar: StatusBar::default(),
            input_dialog: None,
            stats: Stats::default(),
            protocol_handler: handler,
            server_to_ui_rx: Some(server_to_ui_rx),
            args,
        };

        Ok(app)
    }

    pub fn receive_message(&mut self) {
        if let Some(server_to_ui_rx) = self.server_to_ui_rx.as_mut() {
            match server_to_ui_rx.try_recv() {
                core::result::Result::Ok(message) => {
                    let conn_info = message.connection_info.as_ref();
                    let conn_id = conn_info.map(|c| c.connection_id.clone());
                    let conn_addr = conn_info.map(|c| c.remote_addr.to_string());

                    match message.content {
                        common::MessageType::Text(txt) => {
                            self.add_received_message(
                                self.format_header(conn_addr.as_deref()),
                                txt,
                                conn_id.as_deref(),
                            );
                        }
                        common::MessageType::ClientConnected => {
                            self.set_connected(true);
                            let id = conn_id.unwrap();
                            self.receive_view.add_connection(&id);
                            self.send_view.add_connection(&id);
                        }
                        common::MessageType::ClientDisconnected => {
                            self.set_connected(false);
                            let id = conn_id.unwrap();
                            self.receive_view.close_connection_by_title(&id);
                            self.send_view.close_connection_by_title(&id);
                        }
                        common::MessageType::Binary(data) => {
                            let hex_str = crate::utils::data_format::bytes_to_hex(&data);
                            self.add_received_message(
                                self.format_header(conn_addr.as_deref()),
                                format!("[Binary] {}", hex_str),
                                conn_id.as_deref(),
                            );
                        }
                        common::MessageType::Hex(hex_str) => {
                            self.add_received_message(
                                self.format_header(conn_addr.as_deref()),
                                format!("[Hex] {}", hex_str),
                                conn_id.as_deref(),
                            );
                        }
                    }
                }
                core::result::Result::Err(_) => {}
            }
        }
    }

    fn format_header(&self, addr: Option<&str>) -> String {
        let ts = chrono::Local::now().format("%H:%M:%S");
        match addr {
            Some(a) => format!("{} | {}", ts, a),
            None => format!("{}", ts),
        }
    }

    /// 处理按键事件
    pub fn handle_key_event(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match self.input_mode {
            InputMode::Normal => self.handle_normal_mode_key(key, modifiers),
            InputMode::Editing => self.handle_editing_mode_key(key, modifiers),
        }
    }

    fn handle_normal_mode_key(&mut self, key: KeyCode, modifiers: KeyModifiers) -> Result<()> {
        match (key, modifiers) {
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.should_quit = true;
            }

            // I 键打开输入对话框（HTTP 系列协议不支持）
            (KeyCode::Char('i'), KeyModifiers::NONE) => {
                if protocol_supports_send(&self.args.protocol) {
                    let mut dialog = InputDialog::new();
                    // 填充已连接客户端列表
                    let connections = self.protocol_handler.get_connections();
                    for conn in connections {
                        dialog.add_client(conn.connection_id);
                    }
                    self.input_mode = InputMode::Editing;
                    self.input_dialog = Some(dialog);
                }
            }

            // Tab 键切换接收区 tab
            (KeyCode::Tab, KeyModifiers::NONE) => {
                self.receive_view.next_tab();
            }
            (KeyCode::BackTab, KeyModifiers::NONE) => {
                self.receive_view.prev_tab();
            }

            // Shift+左右箭头切换发送区/接收区 tab
            (KeyCode::Left, KeyModifiers::SHIFT) => {
                self.send_view.prev_tab();
            }
            (KeyCode::Right, KeyModifiers::SHIFT) => {
                self.send_view.next_tab();
            }

            _ => {}
        }
        Ok(())
    }

    fn handle_editing_mode_key(&mut self, key: KeyCode, _modifiers: KeyModifiers) -> Result<()> {
        if let Some(dialog) = &mut self.input_dialog {
            match key {
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.input_dialog = None;
                }
                KeyCode::Enter => {
                    if let Some(result) = dialog.submit() {
                        self.send_message_from_dialog(result.input, result.format_hex, result.target_client);
                    }
                    self.input_mode = InputMode::Normal;
                    self.input_dialog = None;
                }
                KeyCode::Tab => {
                    dialog.toggle_format();
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

    fn send_message_from_dialog(&mut self, input: String, is_hex: bool, target_client: Option<String>) {
        let message_type = if is_hex {
            match crate::utils::data_format::hex_to_bytes(&input) {
                Ok(_bytes) => common::MessageType::Hex(input.clone()),
                Err(_) => common::MessageType::Text(input.clone()),
            }
        } else {
            common::MessageType::Text(input.clone())
        };

        let data_len = input.len();
        self.stats.sent_bytes += data_len;
        self.stats.last_activity = Instant::now();

        let header = self.format_header(target_client.as_deref());
        self.send_view.add_message(header, input, target_client.as_deref());

        if let Some(tx) = self.protocol_handler.get_ui_to_server_sender() {
            let msg = common::Message {
                content: message_type,
                direction: common::MessageDirection::Sent,
                timestamp: chrono::Local::now(),
                connection_info: None,
            };
            tokio::spawn(async move {
                let _ = tx.send(msg).await;
            });
        }
    }

    pub fn add_received_message(&mut self, header: String, content: String, tab: Option<&str>) {
        self.stats.received_bytes += content.len();
        self.stats.last_activity = Instant::now();
        self.receive_view.add_message(header, content, tab);
    }

    pub fn set_connected(&mut self, connected: bool) {
        self.stats.connected = connected;
    }
}
