use axum::{
    extract::ws::{Message, WebSocket, WebSocketUpgrade},
    response::{Html, IntoResponse},
    routing::get,
    serve, Router,
};
use std::time::Duration;
use sysinfo::System;
use tokio::time::sleep;
use tower_http::services::ServeDir;

async fn ws(ws: WebSocketUpgrade) -> impl IntoResponse {
    ws.on_upgrade(handle_socket)
}

async fn handle_socket(mut socket: WebSocket) {
    let mut sys = System::new_all();

    loop {
        sys.refresh_all();
        let update = format!(
            r#"{{"cpu_usage": {:.2}, "total_memory": {}, "used_memory": {}}}"#,
            sys.global_cpu_usage(),
            sys.total_memory(),
            sys.used_memory()
        );

        if socket.send(Message::Text(update.into())).await.is_err() {
            break;
        }

        sleep(Duration::from_secs(1)).await;
    }
}

async fn root() -> Html<String> {
    Html(std::include_str!("../templates/index.html").into())
}

#[tokio::main]
async fn main() {
    twink::log::setup();

    let app = Router::new() //
        .route("/", get(root))
        .route("/ws", get(ws))
        .nest_service("/static", ServeDir::new("./static"));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    log::info!("server running @ 0.0.0.0:3000");
    serve(listener, app).await.unwrap();
}
