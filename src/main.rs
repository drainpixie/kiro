use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo,
    },
    response::{Html, IntoResponse},
    routing::get,
    serve, Router,
};
use log::{debug, error, info};
use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use sysinfo::System;
use tokio::{net::UdpSocket, time::sleep};
use tower_http::services::ServeDir;

async fn get_local_ip() -> Result<IpAddr, std::io::Error> {
    debug!("fetching local ip");

    let socket = UdpSocket::bind("0.0.0.0:0").await?;
    socket.connect("8.8.8.8:80").await?;
    let ip = socket.local_addr()?.ip();

    debug!("fetched local ip {}", ip);
    Ok(ip)
}

async fn ws_handler(
    ws: WebSocketUpgrade,
    ConnectInfo(addr): ConnectInfo<std::net::SocketAddr>,
) -> impl IntoResponse {
    debug!("websocket connection from {}", addr);
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, addr: std::net::SocketAddr) {
    debug!("handling socket from {}", addr);

    let mut sys = System::new_all();
    let host_name = System::host_name().unwrap_or_else(|| "unknown".to_string());
    let local_ip = match get_local_ip().await {
        Ok(ip) => ip.to_string(),
        Err(err) => {
            error!("failed to get local IP: {}", err);
            "unknown".to_string()
        }
    };

    debug!("host_name {}", host_name);
    debug!("local_ip {}", local_ip);

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

        debug!("sending update to {}: {}", addr, update);
        if socket
            .send(Message::Text(update.to_string().into()))
            .await
            .is_err()
        {
            error!("failed to send message to {}", addr);
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
    twink::log::setup_level(log::LevelFilter::Debug);

    debug!("starting server");
    let app = Router::new()
        .route("/", get(root))
        .route("/ws", get(ws_handler))
        .nest_service("/static", ServeDir::new("./static"));

    let addr = "127.0.0.1:3000";
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind server");

    info!("server running at {}", addr);
    serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("server error");
}
