use actix_cors::Cors;
use actix_web::{web, App, HttpResponse, HttpServer, Responder};
use rosc::{OscMessage, OscPacket, OscType};
use std::net::{IpAddr, Ipv4Addr, UdpSocket};
use if_addrs::get_if_addrs;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // 获取内网IP地址
    let local_ip = get_local_ip().expect("Failed to get local IP address");
    let local_ip_str = local_ip.to_string();

    println!("Local IP address: {}", local_ip_str);

    // 启动HTTP服务器，绑定到内网IP地址
    HttpServer::new(|| {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .route("/send", web::post().to(send_to_osc))
    })
        .bind(format!("{}:18080", local_ip_str))?
        .run()
        .await
}

// 判断是否是私有 IPv4 地址
fn is_private_ipv4(ip: Ipv4Addr) -> bool {
    ip.is_private() || ip.is_loopback()
}

// 获取内网IP地址
fn get_local_ip() -> Result<IpAddr, String> {
    for interface in get_if_addrs().map_err(|e| e.to_string())? {
        if let IpAddr::V4(ip) = interface.ip() {
            if !ip.is_loopback() && is_private_ipv4(ip) {
                return Ok(std::net::IpAddr::V4(ip));
            }
        }
    }
    Err("没找着ip4地址".to_string())
}
// 处理HTTP请求，将文本发送到OSC端口
async fn send_to_osc(text: web::Json<String>) -> impl Responder {
    // 创建UDP套接字
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind socket");

    // 设置OSC目标地址和端口
    let osc_address = "127.0.0.1:9000";

    // 构建OSC消息
    let msg = OscMessage {
        addr: "/chatbox/input".to_string(), // OSC地址模式
        args: vec![
            OscType::String(text.to_string()), // 文本内容
            OscType::Bool(true),              // 立即发送
            OscType::Bool(false),             // 不触发通知音效
        ],
    };

    // 构建OSC数据包
    let packet = OscPacket::Message(msg);
    let data = rosc::encoder::encode(&packet).expect("Failed to encode packet");

    // 发送数据到OSC端口
    socket.send_to(&data, osc_address).expect("Failed to send data");

    // 返回 HTTP 204 No Content
    HttpResponse::NoContent().finish()
}