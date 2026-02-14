# net-rs

一个基于终端的高性能网络协议调试工具，支持 HTTP/1、HTTP/2、HTTP/3、TCP、UDP、WebSocket 等多种协议。

## 功能特性

### 支持的协议
- **HTTP/1.1** - 完整的 HTTP/1.1 客户端和服务器
- **HTTP/2** - 支持多路复用和服务器推送
- **HTTP/3** - 基于 QUIC 的下一代 HTTP 协议
- **TCP** - 原始 TCP 连接，支持 SSL/TLS
- **UDP** - 无连接数据报协议
- **WebSocket** - 全双工通信协议

### 用户界面
- 基于 [ratatui](https://github.com/ratatui/ratatui) 的现代化终端 UI
- 多标签页支持，可同时管理多个连接
- 实时消息查看和发送
- 可配置的主题和布局

### 其他特性
- 国际化支持（多语言界面）
- TLS/SSL 加密连接支持
- 数据格式自动识别（JSON、XML、Hex 等）
- 连接历史记录和收藏功能
- 导入/导出配置

## 快速开始

### 安装

#### 从源码编译
```bash
# 克隆仓库
git clone https://github.com/punisher1/net-rs.git
cd net-rs

# 编译
cargo build --release

# 运行
./target/release/nt
```

### 使用方法

#### 启动应用程序
```bash
nt
```

#### 使用特定协议
```bash
# HTTP/1.1 模式
nt --protocol http --mode client

# TCP 客户端
nt --protocol tcp --host example.com --port 8080

# WebSocket 连接
nt --protocol websocket --url wss://echo.websocket.org
```

#### 命令行参数
```
nt [OPTIONS]

Options:
  -p, --protocol <PROTOCOL>  指定协议 [http, http2, http3, tcp, udp, websocket]
  -m, --mode <MODE>          运行模式 [client, server]
  -h, --host <HOST>          目标主机地址
  -P, --port <PORT>          目标端口
  -u, --url <URL>            完整的 URL 地址
  -c, --config <CONFIG>      配置文件路径
  -v, --verbose              显示详细日志
  --help                     打印帮助信息
  --version                  打印版本信息
```

## 项目结构

```
net-rs/
├── Cargo.toml          # 项目配置
├── README.md           # 项目说明
├── src/
│   ├── main.rs         # 程序入口
│   ├── app.rs          # 应用状态管理
│   ├── cli/            # 命令行参数解析
│   ├── config/         # 配置管理
│   ├── crossterm.rs    # 终端事件处理
│   ├── protocols/      # 协议实现
│   │   ├── common.rs   # 通用接口
│   │   ├── http.rs     # HTTP/1.1
│   │   ├── http2.rs    # HTTP/2
│   │   ├── http3.rs    # HTTP/3
│   │   ├── tcp.rs      # TCP
│   │   ├── udp.rs      # UDP
│   │   └── websocket.rs# WebSocket
│   ├── ui/             # 用户界面
│   │   ├── layout.rs   # 布局管理
│   │   ├── ui.rs       # UI 渲染
│   │   └── widgets/    # UI 组件
│   └── utils/          # 工具函数
├── locales/            # 国际化文件
├── docs/               # 文档
└── .github/            # GitHub 配置
```

## 开发计划

- [ ] 完善 HTTP/3 实现（QUIC 支持）
- [ ] 添加 gRPC 协议支持
- [ ] 实现数据抓包和保存功能
- [ ] 添加插件系统
- [ ] 支持更多认证方式（OAuth, JWT 等）
- [ ] 添加自动化测试套件
- [ ] 性能优化和基准测试

## 贡献指南

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建你的特性分支 (`git checkout -b feature/AmazingFeature`)
3. 提交你的修改 (`git commit -m 'Add some AmazingFeature'`)
4. 推送到分支 (`git push origin feature/AmazingFeature`)
5. 打开一个 Pull Request

## 许可证

本项目采用 MIT 许可证 - 详见 [LICENSE](LICENSE) 文件。

## 致谢

- [ratatui](https://github.com/ratatui/ratatui) - 优秀的 Rust TUI 库
- [tokio](https://tokio.rs/) - Rust 异步运行时
- [hyper](https://hyper.rs/) - Rust HTTP 实现
- [clap](https://github.com/clap-rs/clap) - 命令行参数解析

## 联系方式

- GitHub: [@punisher1](https://github.com/punisher1)
- 邮箱: your.email@example.com

---

**如果这个项目对你有帮助，请给个 ⭐ Star 支持一下！**
