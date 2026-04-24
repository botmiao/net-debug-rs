# AGENTS.md — net-debug-rs

## 项目概述

`net-debug-rs` 是一个基于终端的网络协议调试工具（TUI），类似 Postman 的命令行版本。使用 Rust + ratatui + tokio 构建。

**二进制名称**: `netd`

## 技术栈

- **UI**: ratatui 0.29 + crossterm 0.29
- **CLI**: clap 4.5 (derive)
- **异步**: tokio 1.45 (full)
- **协议**: tokio (TCP/UDP), tokio-tungstenite (WebSocket), hyper (HTTP), h2 (HTTP/2)
- **TLS**: rustls (ring 后端)
- **错误处理**: anyhow + thiserror
- **i18n**: fluent 0.16

## 构建和测试

```bash
cargo build          # 编译
cargo test           # 运行所有测试 (单元 + 集成)
cargo clippy         # lint 检查
cargo run -- <args>  # 运行
```

## 命令行用法

```bash
netd tcp server 8000                  # TCP 服务器
netd tcp client 127.0.0.1:8000        # TCP 客户端（本地地址自动分配）
netd tcp client 9000 127.0.0.1:8000   # TCP 客户端（指定本地地址）
netd tcps 8000                        # TCP 服务器简写
netd tcpc 127.0.0.1:8000              # TCP 客户端简写
netd --help                           # 帮助
```

## 项目结构

```
src/
├── lib.rs              # 库入口
├── main.rs             # 二进制入口
├── app.rs              # 应用状态管理 (App, InputMode, Stats)
├── crossterm.rs        # 终端事件循环
├── cli/args.rs         # CLI 参数解析 (ProtocolType, AppMode, Args)
├── config/language.rs  # i18n (Fluent)
├── protocols/
│   ├── common.rs       # ProtocolHandler trait, Message, MessageType, 工厂函数
│   ├── tcp.rs          # TCP server/client (已实现)
│   ├── udp.rs          # UDP (stub)
│   ├── websocket.rs    # WebSocket (stub)
│   ├── http.rs         # HTTP (stub)
│   ├── http2.rs        # HTTP/2 (stub)
│   └── http3.rs        # HTTP/3 (stub)
├── ui/
│   ├── ui.rs           # 主 draw 函数
│   ├── layout.rs       # AppLayout
│   └── widgets/
│       ├── status_bar.rs      # 顶部/底部状态栏
│       ├── message_view.rs    # 消息列表 + tab
│       ├── input_dialog.rs    # 发送消息对话框
│       └── tabs.rs            # TabsState 管理
└── utils/data_format.rs       # hex/string/json 转换
```

## 关键类型

- `ProtocolHandler` (async trait) — 协议处理器接口，定义 start/stop/send_message/get_connections 等
- `MessageType` — Text/Binary/Hex/ClientConnected/ClientDisconnected
- `Message` — content + direction + timestamp + connection_info
- `App` — 主应用状态，持有 ProtocolHandler、消息视图、输入对话框
- `Args` — CLI 解析结果，包含 ProtocolType + AppMode + 地址

## 开发约定

- 代码注释使用中文
- 协议处理器通过 `create_protocol_handler()` 工厂函数创建
- `std::sync::RwLock` 用于需要在同步方法中访问的共享状态（避免 block_on）
- `tokio::sync::mpsc::channel` 用于协议层到 UI 层的消息传递
- UI 使用 ratatui 的 immediate mode 模式，每帧重绘

## 实现状态

| 协议 | 状态 |
|------|------|
| TCP Server | 已实现 |
| TCP Client | 已实现 |
| UDP Server/Client | Stub |
| WebSocket Server/Client | Stub |
| HTTP/HTTP2/HTTP3 | Stub |

## 快捷键

- `Ctrl+C` — 退出
- `I` — 打开消息输入对话框（HTTP 系列协议不可用）
- `Tab` — 切换接收区 tab
- `Shift+←/→` — 切换发送区 tab
- 输入对话框内 `Tab` — 切换 String/Hex 格式
- 终端 `Ctrl+Shift+V` — 粘贴（终端原生支持）

## UI 设计

- 消息显示为两行：第一行 `── 时间戳 | 连接地址 ──`（灰色分隔线），第二行为实际内容
- 发送区和接收区都有 per-client tab，tab 标题为客户端 IP:Port
- HTTP/HTTP2/HTTP3 协议不响应 I 键（短连接模式）
- 输入对话框支持选择目标客户端和数据格式（String/Hex）
