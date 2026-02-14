use anyhow::Result;
use async_trait::async_trait;
use bytes::Bytes;
use std::{collections::HashMap, net::SocketAddr, sync::Arc};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::{
        mpsc::{channel, Receiver, Sender},
        RwLock,
    },
};

use crate::protocols::common::{ConnectionInfo, Message, MessageDirection, MessageType, ProtocolHandler};

// 简单的十六进制解码辅助函数
fn hex_decode(hex_str: &str) -> Option<Vec<u8>> {
    let hex_str = hex_str.trim();
    if hex_str.is_empty() {
        return Some(Vec::new());
    }
    
    // 移除可能的 0x 前缀
    let hex_str = hex_str.strip_prefix("0x").unwrap_or(hex_str);
    let hex_str = hex_str.strip_prefix("0X").unwrap_or(hex_str);
    
    // 移除所有空白字符
    let hex_str: String = hex_str.chars().filter(|c| !c.is_whitespace()).collect();
    
    if hex_str.len() % 2 != 0 {
        return None;
    }
    
    let mut result = Vec::with_capacity(hex_str.len() / 2);
    for i in (0..hex_str.len()).step_by(2) {
        let byte_str = &hex_str[i..i + 2];
        match u8::from_str_radix(byte_str, 16) {
            Ok(byte) => result.push(byte),
            Err(_) => return None,
        }
    }
    
    Some(result)
}

/// TCP 服务器处理器
pub struct TcpServerHandler {
    /// 本地地址
    local_addr: SocketAddr,
    /// 连接的客户端
    clients: Arc<RwLock<HashMap<String, TcpClientInfo>>>,
    /// 控制通道 (用于停止服务器)
    control_tx: Option<Sender<()>>,
    /// UI到服务器发送通道
    ui_to_server_tx: Option<Sender<Message>>,
    /// UI到服务器接收通道
    ui_to_server_rx: Option<Receiver<Message>>,
    /// 服务器到UI发送通道
    server_to_ui_tx: Option<Sender<Message>>,
    /// 运行状态
    running: bool,
}

/// TCP 客户端信息
struct TcpClientInfo {
    /// 远程地址
    addr: SocketAddr,
    /// 发送通道
    tx: Sender<Bytes>,
}

impl TcpServerHandler {
    /// 创建新的TCP服务器处理器
    pub fn new(local_addr: SocketAddr) -> Self {
        Self {
            local_addr,
            clients: Arc::new(RwLock::new(HashMap::new())),
            control_tx: None,
            ui_to_server_tx: None,
            ui_to_server_rx: None,
            server_to_ui_tx: None,
            running: false,
        }
    }
}

#[async_trait]
impl ProtocolHandler for TcpServerHandler {
    async fn start(&mut self) -> Result<()> {
        // 创建消息通道
        let (ui_to_server_tx, ui_to_server_rx) = channel::<Message>(100);
        let (control_tx, mut control_rx) = channel::<()>(1);

        self.ui_to_server_tx = Some(ui_to_server_tx);
        self.ui_to_server_rx = Some(ui_to_server_rx);
        self.control_tx = Some(control_tx);
        self.running = true;

        // 绑定监听地址
        let listener = TcpListener::bind(self.local_addr).await?;

        let clients = Arc::clone(&self.clients);
        let server_to_ui_tx = self.server_to_ui_tx.clone();

        // 启动服务器监听任务
        tokio::spawn(async move {
            loop {
                tokio::select! {
                    // 处理新的客户端连接
                    result = listener.accept() => {
                        match result {
                            Ok((stream, addr)) => {
                                // 为每个客户端创建处理任务
                                let client_id = addr.to_string();
                                let (client_tx, mut client_rx) = channel::<Bytes>(100);

                                // 保存客户端信息
                                {
                                    let mut clients_lock = clients.write().await;
                                    clients_lock.insert(client_id.clone(), TcpClientInfo {
                                        addr,
                                        tx: client_tx,
                                    });
                                }

                                // 通知UI有新连接
                                if let Some(ref server_to_ui_sender) = server_to_ui_tx {
                                    let _ = server_to_ui_sender.send(Message {
                                        content: MessageType::ClientConnected,
                                        direction: MessageDirection::Received,
                                        timestamp: chrono::Local::now(),
                                        connection_info: Some(ConnectionInfo {
                                           remote_addr: addr,
                                           connection_id: client_id.clone(),
                                        }),
                                    }).await;
                                }

                                // 分离读写流
                                let (mut read_half, mut write_half) = stream.into_split();
                                let clients_for_read = Arc::clone(&clients);
                                let server_to_ui_tx_for_read = server_to_ui_tx.clone();

                                // 处理客户端读取任务
                                let read_client_id = client_id.clone();
                                tokio::spawn(async move {
                                    let mut buffer = vec![0u8; 4096];
                                    loop {
                                        match read_half.read(&mut buffer).await {
                                            Ok(0) => {
                                                // 从客户端列表中移除
                                                {
                                                    let mut clients_lock = clients_for_read.write().await;
                                                    clients_lock.remove(&read_client_id);
                                                }

                                                // 通知UI连接断开
                                                if let Some(ref server_to_ui_sender) = server_to_ui_tx_for_read {
                                                    let _ = server_to_ui_sender.send(Message {
                                                        direction: MessageDirection::Received,
                                                        content: MessageType::ClientDisconnected,
                                                        timestamp: chrono::Local::now(),
                                                        connection_info: Some(ConnectionInfo {
                                                            remote_addr: addr,
                                                            connection_id: read_client_id.clone(),
                                                        }),
                                                    }).await;
                                                }
                                                break;
                                            }
                                            Ok(n) => {
                                                // 接收到数据
                                                let data = &buffer[..n];
                                                let message_content = String::from_utf8_lossy(data).to_string();

                                                // 发送到UI
                                                if let Some(ref server_to_ui_sender) = server_to_ui_tx_for_read {
                                                    let _ = server_to_ui_sender.send(Message {
                                                        direction: MessageDirection::Received,
                                                        content: MessageType::Text(message_content.clone()),
                                                        timestamp: chrono::Local::now(),
                                                        connection_info: Some(ConnectionInfo {
                                                            remote_addr: addr,
                                                            connection_id: read_client_id.clone(),
                                                        }),
                                                    }).await;
                                                }
                                            }
                                            Err(e) => {
                                                println!("读取客户端 {} 数据时出错: {}", addr, e);
                                                break;
                                            }
                                        }
                                    }

                                    drop(read_half);
                                });

                                // 处理客户端写入任务
                                tokio::spawn(async move {
                                    while let Some(data) = client_rx.recv().await {
                                        if let Err(e) = write_half.write_all(&data).await {
                                            println!("向客户端 {} 发送数据时出错: {}", addr, e);
                                            break;
                                        }
                                    }

                                    drop(write_half);
                                });
                            }
                            Err(e) => {
                                println!("接受客户端连接时出错: {}", e);
                            }
                        }
                    }

                    // 处理停止信号 - 明确检查是否收到了信号
                    result = control_rx.recv() => {
                        match result {
                            Some(_) => {
                                println!("TCP 服务器收到停止信号，正在停止...");
                                break;
                            }
                            None => {
                                // 发送方已关闭，这通常意味着服务器应该停止
                                println!("TCP 服务器控制通道已关闭，正在停止...");
                                break;
                            }
                        }
                    }
                }
            }
        });

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        if self.running {
            // 发送停止信号
            if let Some(ref control_tx) = self.control_tx {
                let _ = control_tx.send(()).await;
            }
            self.running = false;
            // 清理资源
            self.control_tx = None;
        }
        Ok(())
    }

    async fn send_message(&mut self, message: MessageType, target: Option<String>) -> Result<()> {
        let data = match message {
            MessageType::Text(text) => Bytes::from(text.into_bytes()),
            MessageType::Binary(bytes) => bytes,
            MessageType::Hex(hex_str) => {
                // 将十六进制字符串转换为字节
                let bytes = hex_decode(&hex_str).unwrap_or_default();
                Bytes::from(bytes)
            }
            _ => return Ok(()), // 不支持发送其他类型的消息
        };

        if let Some(target_id) = target {
            // 发送到特定客户端
            let clients = self.clients.read().await;
            if let Some(client) = clients.get(&target_id) {
                let _ = client.tx.send(data).await;
            }
        } else {
            // 广播到所有客户端
            let clients = self.clients.read().await;
            for (_, client) in clients.iter() {
                let _ = client.tx.send(data.clone()).await;
            }
        }

        Ok(())
    }

    fn get_ui_to_server_sender(&self) -> Option<Sender<Message>> {
        self.ui_to_server_tx.clone()
    }

    fn set_server_to_ui_sender(&mut self, sender: Sender<Message>) {
        self.server_to_ui_tx = Some(sender);
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn get_connections(&self) -> Vec<ConnectionInfo> {
        // 使用 block_on 来执行异步代码
        let rt = tokio::runtime::Handle::current();
        rt.block_on(async {
            let clients = self.clients.read().await;
            clients
                .values()
                .map(|client| ConnectionInfo {
                    remote_addr: client.addr,
                    connection_id: client.addr.to_string(),
                })
                .collect()
        })
    }

    fn protocol_name(&self) -> &'static str {
        "TCP Server"
    }
}

/// TCP 客户端处理器
pub struct TcpClientHandler {
    /// 本地地址
    local_addr: SocketAddr,
    /// 远程服务器地址
    remote_addr: SocketAddr,
    /// TCP 流
    stream: Option<TcpStream>,
    /// 控制通道 (用于停止客户端)
    control_tx: Option<Sender<()>>,
    /// 消息接收通道
    ui_to_server_rx: Option<Receiver<Message>>,
    /// 消息发送通道
    ui_to_server_tx: Option<Sender<Message>>,
    /// UI消息发送通道
    server_to_ui_tx: Option<Sender<Message>>,
    /// 运行状态
    running: bool,
}

impl TcpClientHandler {
    /// 创建新的TCP客户端处理器
    pub fn new(local_addr: SocketAddr, remote_addr: SocketAddr) -> Self {
        Self {
            local_addr,
            remote_addr,
            stream: None,
            control_tx: None,
            ui_to_server_rx: None,
            ui_to_server_tx: None,
            server_to_ui_tx: None,
            running: false,
        }
    }
}

#[async_trait]
impl ProtocolHandler for TcpClientHandler {
    async fn start(&mut self) -> Result<()> {
        // 连接到远程服务器
        let stream = TcpStream::connect(self.remote_addr).await?;
        self.stream = Some(stream);
        self.running = true;

        // 创建消息通道
        let (ui_to_server_tx, ui_to_server_rx) = channel::<Message>(100);
        self.ui_to_server_tx = Some(ui_to_server_tx);
        self.ui_to_server_rx = Some(ui_to_server_rx);

        // 获取流的引用用于读取任务
        let stream = self.stream.as_ref().unwrap();
        let (mut read_half, mut write_half) = stream.into_split();
        let server_to_ui_tx = self.server_to_ui_tx.clone();
        let remote_addr = self.remote_addr;

        // 启动读取任务
        tokio::spawn(async move {
            let mut buffer = vec![0u8; 4096];
            loop {
                match read_half.read(&mut buffer).await {
                    Ok(0) => {
                        // 服务器断开连接
                        if let Some(ref tx) = server_to_ui_tx {
                            let _ = tx.send(Message {
                                content: MessageType::ClientDisconnected,
                                direction: MessageDirection::Received,
                                timestamp: chrono::Local::now(),
                                connection_info: Some(ConnectionInfo {
                                    remote_addr,
                                    connection_id: remote_addr.to_string(),
                                }),
                            }).await;
                        }
                        break;
                    }
                    Ok(n) => {
                        let data = &buffer[..n];
                        let message_content = String::from_utf8_lossy(data).to_string();

                        if let Some(ref tx) = server_to_ui_tx {
                            let _ = tx.send(Message {
                                content: MessageType::Text(message_content),
                                direction: MessageDirection::Received,
                                timestamp: chrono::Local::now(),
                                connection_info: Some(ConnectionInfo {
                                    remote_addr,
                                    connection_id: remote_addr.to_string(),
                                }),
                            }).await;
                        }
                    }
                    Err(e) => {
                        println!("读取数据时出错: {}", e);
                        break;
                    }
                }
            }
        });

        // 启动写入任务
        let ui_to_server_rx = self.ui_to_server_rx.take().unwrap();
        tokio::spawn(async move {
            let mut rx = ui_to_server_rx;
            while let Some(msg) = rx.recv().await {
                let data = match msg.content {
                    MessageType::Text(text) => Bytes::from(text.into_bytes()),
                    MessageType::Binary(bytes) => bytes,
                    _ => continue,
                };

                if let Err(e) = write_half.write_all(&data).await {
                    println!("发送数据时出错: {}", e);
                    break;
                }
            }
        });

        // 通知 UI 已连接
        if let Some(ref tx) = self.server_to_ui_tx {
            let _ = tx.send(Message {
                content: MessageType::ClientConnected,
                direction: MessageDirection::Received,
                timestamp: chrono::Local::now(),
                connection_info: Some(ConnectionInfo {
                    remote_addr,
                    connection_id: remote_addr.to_string(),
                }),
            }).await;
        }

        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        self.running = false;
        if let Some(mut stream) = self.stream.take() {
            // 关闭连接
            let _ = stream.shutdown().await;
        }
        Ok(())
    }

    async fn send_message(&mut self, message: MessageType, target: Option<String>) -> Result<()> {
        if let Some(ref tx) = self.ui_to_server_tx {
            let msg = Message {
                content: message,
                direction: MessageDirection::Sent,
                timestamp: chrono::Local::now(),
                connection_info: Some(ConnectionInfo {
                    remote_addr: self.remote_addr,
                    connection_id: self.remote_addr.to_string(),
                }),
            };
            let _ = tx.send(msg).await;
        }
        Ok(())
    }

    fn get_ui_to_server_sender(&self) -> Option<Sender<Message>> {
        self.ui_to_server_tx.clone()
    }

    fn set_server_to_ui_sender(&mut self, sender: Sender<Message>) {
        self.server_to_ui_tx = Some(sender);
    }

    fn is_running(&self) -> bool {
        self.running
    }

    fn get_connections(&self) -> Vec<ConnectionInfo> {
        if self.running {
            vec![ConnectionInfo {
                remote_addr: self.remote_addr,
                connection_id: self.remote_addr.to_string(),
            }]
        } else {
            vec![]
        }
    }

    fn protocol_name(&self) -> &'static str {
        "TCP Client"
    }
}
