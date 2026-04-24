使用rust的ratatui,clap(structopt)构建一个终端的通讯协议调试工具
- 左右或者上下结构，可以通过命令行参数进行控制，分为发送和接收
- 顶部有统计状态：发送，接收，链接状态
- 底部是快捷键提醒
- 多语言支持
- 支持的协议有：tcp、udp、websocket、http、http2、http3、保留：grpc、mqtt、kafka
- 子命令形式如表格所示
- ctrl+c 可退出命令行
- Ctrl+I 可弹出对话框，输入要发送的数据
  - tcp、udp、websocket可选择需要发送的数据类型，string or hex，hex可检测是否是有空格，有空格则split，没有空格则按两个字符一个hex来算
 
 
|                  |                                                                                                                                                                                       |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 命令行              |                                                                                                                                                                                       |
| tcp-server       | //prefix: net tcp server \| net tcps  <br>net tcp server 127.0.0.1:8000net tcp server 8000                                                                                            |
| tcp-client       | //prefix: net tcp client \| net tcpcnet tcp client 127.0.0.1:9000 192.168.0.11:9000net tcp client 9000 192.168.0.11:9000net tcp client 192.168.0.11:9000                              |
| udp-server       | //prefix: net udp server \| net udpsnet udp server 127.0.0.1:8000net udp 8000                                                                                                         |
| udp-client       | //prefix: net udp client \| net udpcnet udp client 127.0.0.1:9000 192.168.0.11:9000net udp client 9000 192.168.0.11:9000net udp client 192.168.0.11:9000                              |
| websocket-server | //prefix: net websocket server \| net ws servernet websocket server 127.0.0.1:8000net websocket server 8000                                                                           |
| websocket-client | //prefix: net websocket client \| net ws clientnet websocket client 127.0.0.1:9000 192.168.0.11:9000net websocket client 9000 192.168.0.11:9000net websocket client 192.168.0.11:9000 |
| http-server      | //prefix: net http servernet http server 127.0.0.1:8000net http server 8000                                                                                                           |
| http-client      | -                                                                                                                                                                                     |
| http2-server     | //prefix: net http2 servernet http2 server 127.0.0.1:8000net http2 server 8000                                                                                                        |
| http2-client     | -                                                                                                                                                                                     |
| http3-server     | //prefix: net http3 servernet http3 server 127.0.0.1:8000net http3 server 8000                                                                                                        |
| http3-client     | -                                                                                                                                                                                     |
| grpc-server      | 保留                                                                                                                                                                                    |
| grpc-client      | 保留                                                                                                                                                                                    |
| mqtt-server      | 保留                                                                                                                                                                                    |
| mqtt-client      | 保留                                                                                                                                                                                    |
| kafka-client     | 保留                                                                                                                                                                                    |

