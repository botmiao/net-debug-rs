use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use chrono::{DateTime, Local};
use std::net::SocketAddr;
use tokio::sync::mpsc::Sender;

use crate::protocols::tcp::TcpServerHandler;

/// 传输消息类型
#[derive(Debug, Clone)]
pub enum MessageType {
    /// 文本消息
    Text(String),
    /// 二进制消息
    Binary(Bytes),
    /// 十六进制消息
    Hex(String),
    /// 客户端连接消息
    ClientConnected,
    /// 客户端断开连接消息
    ClientDisconnected,
}

/// 消息方向
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MessageDirection {
    /// 收到的消息
    Received,
    /// 发送的消息
    Sent,
}

/// 连接信息
#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    /// 远程地址
    pub remote_addr: SocketAddr,
    /// 连接 ID (用于区分不同客户端)
    pub connection_id: String,
}

/// 消息
#[derive(Debug, Clone)]
pub struct Message {
    /// 消息内容
    pub content: MessageType,
    /// 消息方向
    pub direction: MessageDirection,
    /// 时间戳
    pub timestamp: DateTime<Local>,
    /// 连接信息
    pub connection_info: Option<ConnectionInfo>,
}

impl Message {
    /// 创建新的接收消息
    pub fn new_received(content: MessageType, connection_info: Option<ConnectionInfo>) -> Self {
        Self {
            content,
            direction: MessageDirection::Received,
            timestamp: Local::now(),
            connection_info,
        }
    }

    /// 创建新的发送消息
    pub fn new_sent(content: MessageType, connection_info: Option<ConnectionInfo>) -> Self {
        Self {
            content,
            direction: MessageDirection::Sent,
            timestamp: Local::now(),
            connection_info,
        }
    }
}

/// 通讯协议处理接口
#[async_trait]
pub trait ProtocolHandler {
    /// 启动协议处理器
    async fn start(&mut self) -> Result<()>;

    /// 停止协议处理器
    async fn stop(&mut self) -> Result<()>;

    /// 发送消息
    async fn send_message(&mut self, message: MessageType, target: Option<String>) -> Result<()>;

    /// 获取UI向服务端的发送通道
    fn get_ui_to_server_sender(&self) -> Option<Sender<Message>>;

    /// 设置服务端向UI的发送通道
    fn set_server_to_ui_sender(&mut self, sender: Sender<Message>);

    /// 处理程序是否正在运行
    fn is_running(&self) -> bool;

    /// 获取当前连接信息
    fn get_connections(&self) -> Vec<ConnectionInfo>;

    /// 获取协议名称
    fn protocol_name(&self) -> &'static str;
}

/// 创建协议处理器工厂函数
pub async fn create_protocol_handler(
    protocol: &str,
    is_server: bool,
    server_to_ui_tx: Option<Sender<Message>>,
    local_addr: SocketAddr,
    remote_addr: Option<SocketAddr>,
) -> Result<Box<dyn ProtocolHandler + Send + Sync>> {
    match (protocol.to_lowercase().as_str(), is_server) {
        ("tcp", true) => {
            let mut handler = TcpServerHandler::new(local_addr);
            handler.set_server_to_ui_sender(server_to_ui_tx.unwrap());
            handler.start().await?;
            Ok(Box::new(handler))
        }
        ("tcp", false) => {
            // 创建 TCP 客户端处理器
            let remote = remote_addr.ok_or_else(|| anyhow::anyhow!("TCP client requires remote address"))?;
            let mut handler = crate::protocols::tcp::TcpClientHandler::new(local_addr, remote);
            handler.set_server_to_ui_sender(server_to_ui_tx.ok_or_else(|| anyhow::anyhow!("Server to UI sender is required"))?);
            handler.start().await?;
            Ok(Box::new(handler))
        }
        ("udp", true) => {
            anyhow::bail!("UDP server handler not yet implemented")
        }
        ("udp", false) => {
            anyhow::bail!("UDP client handler not yet implemented")
        }
        ("websocket", true) => {
            anyhow::bail!("WebSocket server handler not yet implemented")
        }
        ("websocket", false) => {
            anyhow::bail!("WebSocket client handler not yet implemented")
        }
        ("http", true) => {
            anyhow::bail!("HTTP server handler not yet implemented")
        }
        ("http", false) => {
            anyhow::bail!("HTTP client handler not yet implemented")
        }
        ("http2", true) => {
            anyhow::bail!("HTTP/2 server handler not yet implemented")
        }
        ("http2", false) => {
            anyhow::bail!("HTTP/2 client handler not yet implemented")
        }
        ("http3", true) => {
            anyhow::bail!("HTTP/3 server handler not yet implemented")
        }
        ("http3", false) => {
            anyhow::bail!("HTTP/3 client handler not yet implemented")
        }
        _ => anyhow::bail!("Unsupported protocol: {}", protocol),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::Bytes;

    #[test]
    fn test_message_new_received() {
        let msg = Message::new_received(
            MessageType::Text("hello".to_string()),
            None,
        );
        assert_eq!(msg.direction, MessageDirection::Received);
        assert!(matches!(msg.content, MessageType::Text(ref s) if s == "hello"));
        assert!(msg.connection_info.is_none());
    }

    #[test]
    fn test_message_new_sent() {
        let conn = ConnectionInfo {
            remote_addr: "127.0.0.1:8000".parse().unwrap(),
            connection_id: "test-conn".to_string(),
        };
        let msg = Message::new_sent(
            MessageType::Binary(Bytes::from_static(b"data")),
            Some(conn),
        );
        assert_eq!(msg.direction, MessageDirection::Sent);
        assert!(matches!(msg.content, MessageType::Binary(_)));
        assert!(msg.connection_info.is_some());
    }

    #[test]
    fn test_message_type_variants() {
        let text = MessageType::Text("hello".to_string());
        let binary = MessageType::Binary(Bytes::from_static(b"\x01\x02"));
        let hex = MessageType::Hex("0102FF".to_string());
        let connected = MessageType::ClientConnected;
        let disconnected = MessageType::ClientDisconnected;

        assert!(matches!(text, MessageType::Text(_)));
        assert!(matches!(binary, MessageType::Binary(_)));
        assert!(matches!(hex, MessageType::Hex(_)));
        assert!(matches!(connected, MessageType::ClientConnected));
        assert!(matches!(disconnected, MessageType::ClientDisconnected));
    }

    #[test]
    fn test_message_direction_equality() {
        assert_eq!(MessageDirection::Received, MessageDirection::Received);
        assert_ne!(MessageDirection::Received, MessageDirection::Sent);
    }
}
