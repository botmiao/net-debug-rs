use clap::{Parser, Subcommand, Args as ClapArgs};
use std::net::SocketAddr;

/// 终端网络调试工具
#[derive(Parser, Debug, Clone)]
#[command(name = "netd", author, version, about)]
pub struct Cli {
    /// 使用垂直分割布局 (发送区在上，接收区在下)，默认为水平布局 (左右)
    #[arg(short, long)]
    pub vertical_layout: bool,

    #[command(subcommand)]
    pub command: Commands,
}

/// 支持的协议命令
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    /// TCP 协议
    #[command(subcommand)]
    Tcp(TcpCommands),
    
    /// TCP 服务器模式简写
    #[command(alias = "tcps")]
    TcpServer(ServerArgs),
    
    /// TCP 客户端模式简写
    #[command(alias = "tcpc")]
    TcpClient(ClientArgs),
    
    /// UDP 协议
    #[command(subcommand)]
    Udp(UdpCommands),
    
    /// UDP 服务器模式简写
    #[command(alias = "udps")]
    UdpServer(ServerArgs),
    
    /// UDP 客户端模式简写
    #[command(alias = "udpc")]
    UdpClient(ClientArgs),
    
    /// WebSocket 协议
    #[command(alias = "ws", subcommand)]
    WebSocket(WebSocketCommands),
    
    /// HTTP 协议
    #[command(subcommand)]
    Http(HttpCommands),
    
    /// HTTP/2 协议
    #[command(subcommand)]
    Http2(HttpCommands),
    
    /// HTTP/3 协议
    #[command(subcommand)]
    Http3(HttpCommands),
}

/// TCP 命令
#[derive(Subcommand, Debug, Clone)]
pub enum TcpCommands {
    /// TCP 服务器模式
    #[command(alias = "s")]
    Server(ServerArgs),
    
    /// TCP 客户端模式
    #[command(alias = "c")]
    Client(ClientArgs),
}

/// UDP 命令
#[derive(Subcommand, Debug, Clone)]
pub enum UdpCommands {
    /// UDP 服务器模式
    #[command(alias = "s")]
    Server(ServerArgs),
    
    /// UDP 客户端模式
    #[command(alias = "c")]
    Client(ClientArgs),
}

/// WebSocket 命令
#[derive(Subcommand, Debug, Clone)]
pub enum WebSocketCommands {
    /// WebSocket 服务器模式
    #[command(alias = "s")]
    Server(ServerArgs),
    
    /// WebSocket 客户端模式
    #[command(alias = "c")]
    Client(ClientArgs),
}

/// HTTP 命令 (对于HTTP/1.1, HTTP/2, HTTP/3)
#[derive(Subcommand, Debug, Clone)]
pub enum HttpCommands {
    /// HTTP 服务器模式
    #[command(alias = "s")]
    Server(ServerArgs),
    
    /// HTTP 客户端模式
    #[command(alias = "c")]
    HttpClient(HttpClientArgs),
}

/// 服务器参数
#[derive(ClapArgs, Debug, Clone)]
pub struct ServerArgs {
    /// 服务器地址 (如 127.0.0.1:8000)
    /// 如果只提供端口号则绑定到 127.0.0.1
    pub address: String,
}

/// 客户端参数
#[derive(ClapArgs, Debug, Clone)]
pub struct ClientArgs {
    /// 本地地址 (如 127.0.0.1:9000)
    /// 如果只提供端口号则绑定到 127.0.0.1
    pub local: String,
    
    /// 远程服务器地址 (如 192.168.1.1:8000)
    pub remote: String,
}

/// HTTP 客户端参数
#[derive(ClapArgs, Debug, Clone)]
pub struct HttpClientArgs {
    /// HTTP 请求方法 (GET, POST, PUT 等)
    pub method: String,
    
    /// 目标 URL
    pub url: String,
    
    /// 请求体，适用于 POST/PUT 等方法
    #[arg(short, long)]
    pub body: Option<String>,
    
    /// 请求头，格式为 "Header-Name: Value"
    #[arg(short = 'H', long)]
    pub headers: Vec<String>,
}

/// 命令行参数完整结构
#[derive(Debug, Clone)]
pub struct Args {
    /// 垂直布局标志
    pub vertical_layout: bool,
    
    /// 使用的协议类型
    pub protocol: ProtocolType,
    
    /// 模式 (服务端或客户端)
    pub mode: AppMode,
    
    /// 本地地址
    pub local_addr: SocketAddr,
    
    /// 远程地址 (仅客户端模式)
    pub remote_addr: Option<SocketAddr>,
    
    /// HTTP 特定参数 (仅HTTP协议)
    pub http_args: Option<HttpClientArgs>,
}

/// 协议类型
#[derive(Debug, Clone, PartialEq)]
pub enum ProtocolType {
    Tcp,
    Udp,
    WebSocket,
    Http,
    Http2,
    Http3,
}

/// 应用模式
#[derive(Debug, Clone, PartialEq)]
pub enum AppMode {
    Server,
    Client,
}

/// 解析命令行参数
pub fn parse_args() -> Args {
    let cli = Cli::parse();
    
    // 提取信息，转换成我们的Args结构
    let (protocol, mode, local_addr, remote_addr, http_args) = match &cli.command {
        Commands::Tcp(cmd) => match cmd {
            TcpCommands::Server(args) => {
                (ProtocolType::Tcp, AppMode::Server, parse_address(&args.address), None, None)
            }
            TcpCommands::Client(args) => {
                (ProtocolType::Tcp, AppMode::Client, parse_address(&args.local), Some(parse_address(&args.remote)), None)
            }
        },
        Commands::TcpServer(args) => {
            (ProtocolType::Tcp, AppMode::Server, parse_address(&args.address), None, None)
        },
        Commands::TcpClient(args) => {
            (ProtocolType::Tcp, AppMode::Client, parse_address(&args.local), Some(parse_address(&args.remote)), None)
        },
        Commands::Udp(cmd) => match cmd {
            UdpCommands::Server(args) => {
                (ProtocolType::Udp, AppMode::Server, parse_address(&args.address), None, None)
            }
            UdpCommands::Client(args) => {
                (ProtocolType::Udp, AppMode::Client, parse_address(&args.local), Some(parse_address(&args.remote)), None)
            }
        },
        Commands::UdpServer(args) => {
            (ProtocolType::Udp, AppMode::Server, parse_address(&args.address), None, None)
        },
        Commands::UdpClient(args) => {
            (ProtocolType::Udp, AppMode::Client, parse_address(&args.local), Some(parse_address(&args.remote)), None)
        },
        Commands::WebSocket(cmd) => match cmd {
            WebSocketCommands::Server(args) => {
                (ProtocolType::WebSocket, AppMode::Server, parse_address(&args.address), None, None)
            }
            WebSocketCommands::Client(args) => {
                (ProtocolType::WebSocket, AppMode::Client, parse_address(&args.local), Some(parse_address(&args.remote)), None)
            }
        },
        Commands::Http(cmd) => match cmd {
            HttpCommands::Server(args) => {
                (ProtocolType::Http, AppMode::Server, parse_address(&args.address), None, None)
            }
            HttpCommands::HttpClient(args) => {
                (ProtocolType::Http, AppMode::Client, parse_dummy_addr(), None, Some(args.clone()))
            }
        },
        Commands::Http2(cmd) => match cmd {
            HttpCommands::Server(args) => {
                (ProtocolType::Http2, AppMode::Server, parse_address(&args.address), None, None)
            }
            HttpCommands::HttpClient(args) => {
                (ProtocolType::Http2, AppMode::Client, parse_dummy_addr(), None, Some(args.clone()))
            }
        },
        Commands::Http3(cmd) => match cmd {
            HttpCommands::Server(args) => {
                (ProtocolType::Http3, AppMode::Server, parse_address(&args.address), None, None)
            }
            HttpCommands::HttpClient(args) => {
                (ProtocolType::Http3, AppMode::Client, parse_dummy_addr(), None, Some(args.clone()))
            }
        },
    };

    Args {
        vertical_layout: cli.vertical_layout,
        protocol,
        mode,
        local_addr,
        remote_addr,
        http_args,
    }
}

/// 解析地址字符串，如果只提供端口则使用 127.0.0.1
fn parse_address(addr_str: &str) -> SocketAddr {
    // 检查是否只有端口号
    if let Ok(port) = addr_str.parse::<u16>() {
        format!("127.0.0.1:{}", port).parse().unwrap()
    } else {
        // 尝试直接解析完整地址
        match addr_str.parse() {
            Ok(addr) => addr,
            Err(_) => {
                // 如果解析失败，返回默认地址
                "127.0.0.1:8000".parse().unwrap()
            }
        }
    }
}

/// 为HTTP客户端模式生成一个虚拟地址，因为HTTP客户端不需要绑定到特定地址
fn parse_dummy_addr() -> SocketAddr {
    "127.0.0.1:0".parse().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_address_port_only() {
        let addr = parse_address("8000");
        assert_eq!(addr, "127.0.0.1:8000".parse().unwrap());
    }

    #[test]
    fn test_parse_address_full() {
        let addr = parse_address("192.168.1.1:9000");
        assert_eq!(addr, "192.168.1.1:9000".parse().unwrap());
    }

    #[test]
    fn test_parse_address_invalid_defaults() {
        let addr = parse_address("not_an_address");
        assert_eq!(addr, "127.0.0.1:8000".parse().unwrap());
    }

    #[test]
    fn test_parse_dummy_addr() {
        let addr = parse_dummy_addr();
        assert!(addr.is_ipv4());
        assert_eq!(addr.port(), 0);
    }
}