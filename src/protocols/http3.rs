use anyhow::Result;
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::sync::mpsc::{Receiver, Sender};

use crate::protocols::common::{
    ConnectionInfo, Message, MessageType, ProtocolHandler,
};

/// HTTP/3 服务器处理器
pub struct Http3ServerHandler {
    local_addr: SocketAddr,
    running: bool,
    ui_tx: Option<Sender<Message>>,
    message_tx: Option<Sender<Message>>,
}

impl Http3ServerHandler {
    pub fn new(local_addr: SocketAddr) -> Self {
        Self {
            local_addr,
            running: false,
            ui_tx: None,
            message_tx: None,
        }
    }
}

#[async_trait]
impl ProtocolHandler for Http3ServerHandler {
    async fn start(&mut self) -> Result<()> {
        anyhow::bail!("HTTP/3 server handler not yet fully implemented")
    }
    
    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }
    
    async fn send_message(&mut self, _message: MessageType, _target: Option<String>) -> Result<()> {
        anyhow::bail!("HTTP/3 server send not yet implemented")
    }
    
    fn get_ui_to_server_sender(&self) -> Option<Sender<Message>> {
        self.message_tx.clone()
    }
    
    fn set_server_to_ui_sender(&mut self, sender: Sender<Message>) {
        self.ui_tx = Some(sender);
    }
    
    fn is_running(&self) -> bool {
        self.running
    }
    
    fn get_connections(&self) -> Vec<ConnectionInfo> {
        vec![]
    }
    
    fn protocol_name(&self) -> &'static str {
        "HTTP/3 Server"
    }
}

/// HTTP/3 客户端处理器
pub struct Http3ClientHandler {
    running: bool,
    ui_tx: Option<Sender<Message>>,
    message_tx: Option<Sender<Message>>,
}

impl Http3ClientHandler {
    pub fn new() -> Self {
        Self {
            running: false,
            ui_tx: None,
            message_tx: None,
        }
    }
}

#[async_trait]
impl ProtocolHandler for Http3ClientHandler {
    async fn start(&mut self) -> Result<()> {
        anyhow::bail!("HTTP/3 client handler not yet fully implemented")
    }
    
    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        Ok(())
    }
    
    async fn send_message(&mut self, _message: MessageType, _target: Option<String>) -> Result<()> {
        anyhow::bail!("HTTP/3 client send not yet implemented")
    }
    
    fn get_ui_to_server_sender(&self) -> Option<Sender<Message>> {
        self.message_tx.clone()
    }
    
    fn set_server_to_ui_sender(&mut self, sender: Sender<Message>) {
        self.ui_tx = Some(sender);
    }
    
    fn is_running(&self) -> bool {
        self.running
    }
    
    fn get_connections(&self) -> Vec<ConnectionInfo> {
        vec![]
    }
    
    fn protocol_name(&self) -> &'static str {
        "HTTP/3 Client"
    }
}