use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpStream;
use tokio::time::timeout;

use net_debug_rs::protocols::common::{
    create_protocol_handler, Message, MessageDirection, MessageType,
};

/// 测试 TCP server 能启动并监听端口
#[tokio::test]
async fn test_tcp_server_starts() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_addr = listener.local_addr().unwrap();
    drop(listener);

    let (tx, _rx) = tokio::sync::mpsc::channel(100);
    let mut handler = create_protocol_handler("tcp", true, Some(tx), server_addr, None)
        .await
        .expect("Failed to create TCP server handler");

    assert!(handler.is_running());
    assert_eq!(handler.protocol_name(), "TCP Server");

    handler.stop().await.unwrap();
}

/// 测试 TCP server 接受客户端连接并接收消息
#[tokio::test]
async fn test_tcp_server_receives_message() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_addr = listener.local_addr().unwrap();
    drop(listener);

    let (server_tx, mut server_rx) = tokio::sync::mpsc::channel(100);
    let mut server = create_protocol_handler("tcp", true, Some(server_tx), server_addr, None)
        .await
        .expect("Failed to create server");

    // 客户端连接并发送消息
    let mut stream = TcpStream::connect(server_addr)
        .await
        .expect("Failed to connect to server");

    stream.write_all(b"hello server").await.unwrap();

    // 等待 ClientConnected 消息
    let msg = timeout(Duration::from_secs(2), server_rx.recv())
        .await
        .expect("Timeout waiting for message")
        .expect("Channel closed");
    assert!(matches!(msg.content, MessageType::ClientConnected));

    // 等待数据消息
    let msg = timeout(Duration::from_secs(2), server_rx.recv())
        .await
        .expect("Timeout waiting for data message")
        .expect("Channel closed");
    assert!(matches!(msg.content, MessageType::Text(ref s) if s == "hello server"));

    server.stop().await.unwrap();
}

/// 测试 TCP client handler 连接远程 server
#[tokio::test]
async fn test_tcp_client_connects() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_addr = listener.local_addr().unwrap();

    // 后台接受连接
    let accept_handle = tokio::spawn(async move {
        let (stream, _addr) = listener.accept().await.unwrap();
        stream
    });

    // 创建 client
    let client_local: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let (client_tx, mut client_rx) = tokio::sync::mpsc::channel(100);
    let mut client = create_protocol_handler(
        "tcp",
        false,
        Some(client_tx),
        client_local,
        Some(server_addr),
    )
    .await
    .expect("Failed to create client");

    assert!(client.is_running());
    assert_eq!(client.protocol_name(), "TCP Client");

    // client 应该发送了 ClientConnected 消息
    let msg = timeout(Duration::from_secs(2), client_rx.recv())
        .await
        .expect("Timeout")
        .expect("Channel closed");
    assert!(matches!(msg.content, MessageType::ClientConnected));

    let _server_stream = accept_handle.await.unwrap();
    client.stop().await.unwrap();
}

/// 测试 server 向 client 发送数据
#[tokio::test]
async fn test_tcp_server_send_to_client() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_addr = listener.local_addr().unwrap();
    drop(listener);

    let (server_tx, mut server_rx) = tokio::sync::mpsc::channel(100);
    let mut server = create_protocol_handler("tcp", true, Some(server_tx), server_addr, None)
        .await
        .unwrap();

    // 客户端连接
    let mut stream = TcpStream::connect(server_addr).await.unwrap();

    // 等待 ClientConnected
    let msg = timeout(Duration::from_secs(2), server_rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(msg.content, MessageType::ClientConnected));

    let client_id = msg.connection_info.unwrap().connection_id;

    // server 发送消息给客户端
    server
        .send_message(
            MessageType::Text("hello client".to_string()),
            Some(client_id),
        )
        .await
        .unwrap();

    // client 读取
    let mut buf = vec![0u8; 1024];
    let n = timeout(Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(&buf[..n], b"hello client");

    server.stop().await.unwrap();
}

/// 测试 TCP server 会消费 UI 发送通道并转发给目标客户端
#[tokio::test]
async fn test_tcp_server_ui_sender_sends_to_target_client() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_addr = listener.local_addr().unwrap();
    drop(listener);

    let (server_tx, mut server_rx) = tokio::sync::mpsc::channel(100);
    let mut server = create_protocol_handler("tcp", true, Some(server_tx), server_addr, None)
        .await
        .unwrap();

    let mut stream = TcpStream::connect(server_addr).await.unwrap();

    let msg = timeout(Duration::from_secs(2), server_rx.recv())
        .await
        .unwrap()
        .unwrap();
    assert!(matches!(msg.content, MessageType::ClientConnected));
    let conn = msg.connection_info.unwrap();

    let ui_tx = server.get_ui_to_server_sender().unwrap();
    ui_tx
        .send(Message {
            content: MessageType::Text("来自服务端".to_string()),
            direction: MessageDirection::Sent,
            timestamp: chrono::Local::now(),
            connection_info: Some(conn),
        })
        .await
        .unwrap();

    let mut buf = vec![0u8; 1024];
    let n = timeout(Duration::from_secs(2), stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(std::str::from_utf8(&buf[..n]).unwrap(), "来自服务端");

    server.stop().await.unwrap();
}

/// 测试 TCP client 会通过 UI 发送通道发送 Hex 数据
#[tokio::test]
async fn test_tcp_client_ui_sender_sends_hex() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let server_addr = listener.local_addr().unwrap();

    let accept_handle = tokio::spawn(async move {
        let (stream, _addr) = listener.accept().await.unwrap();
        stream
    });

    let client_local: std::net::SocketAddr = "127.0.0.1:0".parse().unwrap();
    let (client_tx, mut client_rx) = tokio::sync::mpsc::channel(100);
    let client = create_protocol_handler(
        "tcp",
        false,
        Some(client_tx),
        client_local,
        Some(server_addr),
    )
    .await
    .expect("Failed to create client");

    let _ = timeout(Duration::from_secs(2), client_rx.recv())
        .await
        .unwrap()
        .unwrap();
    let mut server_stream = accept_handle.await.unwrap();

    let ui_tx = client.get_ui_to_server_sender().unwrap();
    ui_tx
        .send(Message {
            content: MessageType::Hex("E4 B8 AD".to_string()),
            direction: MessageDirection::Sent,
            timestamp: chrono::Local::now(),
            connection_info: None,
        })
        .await
        .unwrap();

    let mut buf = vec![0u8; 16];
    let n = timeout(Duration::from_secs(2), server_stream.read(&mut buf))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(&buf[..n], "中".as_bytes());
}
