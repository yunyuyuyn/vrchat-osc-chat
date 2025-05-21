// main.rs
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

    println!("Server running at {}:18080", local_ip_str);
    // 启动HTTP服务器
    HttpServer::new(|| {
        // 配置CORS
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header();

        App::new()
            .wrap(cors)
            .route("/send", web::post().to(send_to_osc))
    })
        .bind((local_ip, 18080))?
        .run()
        .await
}

// 判断私有IPv4地址（使用标准库方法）
fn is_private_ipv4(ip: Ipv4Addr) -> bool {
    ip.is_private() || ip.is_loopback()
}
// 获取内网IP地址
fn get_local_ip() -> Result<IpAddr, String> {
    for interface in get_if_addrs().map_err(|e| e.to_string())? {
        if let IpAddr::V4(ip) = interface.ip() {
            if !ip.is_loopback() && is_private_ipv4(ip) {
                return Ok(IpAddr::V4(ip));
            }
        }
    }
    Err("没找着ip4".to_string())
}
// OSC消息处理

async fn send_to_osc(text: web::Json<String>) -> impl Responder {
    //预处理
    let message = text.into_inner();
    println!("已接收:{}",&message);
    // 创建UDP套接字
    let socket = UdpSocket::bind("0.0.0.0:0").expect("Failed to bind UDP socket");
    // OSC目标地址
    let target_addr = "127.0.0.1:9000";
    // 构建OSC消息
    let msg = OscMessage {
        addr: "/chatbox/input".to_string(),
        args: vec![
            OscType::String(message),
            OscType::Bool(true),
            OscType::Bool(false),
        ],
    };

    // 编码并发送
    let packet = OscPacket::Message(msg);
    let buf = rosc::encoder::encode(&packet).expect("OSC encoding failed");
    socket.send_to(&buf, target_addr).expect("Failed to send OSC message");
    HttpResponse::NoContent().finish()
}