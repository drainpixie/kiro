use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::{Html, IntoResponse},
    routing::get,
    serve, Router,
};
use log::{error, info};
use std::{net::IpAddr, time::Duration};
use sysinfo::System;
use tokio::{net::UdpSocket, time::sleep};
use tower_http::services::ServeDir;

async fn get_local_ip() -> Result<IpAddr, std::io::Error> {
    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect("8.8.8.8:80").await?;
    socket.local_addr().map(|addr| addr.ip())
}

async fn ws_handler(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let mut sys = System::new_all();
    let host_name = System::host_name().unwrap_or_else(|| "unknown".to_string());
    let local_ip = match get_local_ip().await {
        Ok(ip) => ip.to_string(),
        Err(err) => {
            error!("failed to get local IP: {}", err);
            "unknown".to_string()
        }
    };

    loop {
        sys.refresh_cpu_all();
        sys.refresh_memory();

        let update = serde_json::json!({
            "cpu_usage": sys.global_cpu_usage(),
            "total_memory": sys.total_memory(),
            "used_memory": sys.used_memory(),
            "system_name": host_name,
            "ip": local_ip
        });

        if socket
            .send(Message::Text(update.to_string().into()))
            .await
            .is_err()
        {
            break;
        }

        sleep(Duration::from_secs(1)).await;
    }
}

async fn root() -> Html<&'static str> {
    Html(include_str!("../templates/index.html"))
}

#[tokio::main]
async fn main() {
    twink::log::setup();

    let app = Router::new()
        .route("/", get(root))
        .route("/ws", get(ws_handler))
        .nest_service("/static", ServeDir::new("./static"));

    let addr = "0.0.0.0:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind server");

    info!("server running @ {}", addr);
    serve(listener, app).await.expect("server error");
}
