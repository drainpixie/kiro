use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        ConnectInfo,
    },
    response::IntoResponse,
    routing::get,
    serve, Router,
};
use flexbuffers::Builder;
use log::{debug, error, info};
use std::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use sysinfo::{MemoryRefreshKind, System};

const POLL_TIME: Duration = Duration::from_secs(1);
const ADDRESS: &str = "127.0.0.1:3000";

use tokio::{net::UdpSocket, time::sleep};

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
    let os = System::long_os_version().unwrap_or_default();
    let kernel_version = System::kernel_version().unwrap_or_default();
    let host_name = System::host_name().unwrap_or_default();
    let local_ip = get_local_ip().await.map_or_else(
        |err| {
            error!("failed to get local IP: {}", err);
            "unknown".to_string()
        },
        |ip| ip.to_string(),
    );

    debug!("host_name {}", host_name);
    debug!("local_ip {}", local_ip);

    loop {
        sys.refresh_cpu_usage();
        sys.refresh_memory_specifics(MemoryRefreshKind::nothing().with_ram());

        let mut builder = Builder::default();
        {
            let mut map = builder.start_map();
            map.push("os", os.as_str());
            map.push("uptime", System::uptime());
            map.push("local_ip", local_ip.as_str());
            map.push("host_name", host_name.as_str());
            map.push("used_memory", sys.used_memory());
            map.push("free_memory", sys.free_memory());
            map.push("kernel", kernel_version.as_str());
            map.push("total_memory", sys.total_memory());
            map.push("cpu_usage", sys.global_cpu_usage());
        }

        let data = builder.view().to_vec();

        debug!("sending {:?} bytes update to {}", data.len(), addr);
        if socket.send(Message::Binary(data.into())).await.is_err() {
            error!("failed to send message to {}", addr);
            break;
        }

        sleep(POLL_TIME).await;
    }
}

#[tokio::main]
async fn main() {
    twink::log::setup_level(log::LevelFilter::Debug);

    debug!("starting server");

    let app = Router::new().route("/", get(ws_handler));
    let listener = tokio::net::TcpListener::bind(ADDRESS)
        .await
        .expect("failed to bind server");

    info!("server running at {}", ADDRESS);
    serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .await
    .expect("server error");
}
