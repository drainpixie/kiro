use futures_util::StreamExt;
use log::{error, kv::Error};
use std::{
    collections::HashMap,
    sync::{mpsc, Arc, Mutex},
};
use tokio_tungstenite::connect_async;
use url::Url;

pub struct WebSocketManager<'a> {
    connections: Arc<Mutex<HashMap<&'a str, mpsc::Sender<&'a str>>>>,

    // TODO: Message struct is just `kiro::Node`, port it here.
    messages: Arc<Mutex<HashMap<&'a str, Vec<&'a str>>>>,
}

impl<'a> WebSocketManager<'a> {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(Mutex::new(HashMap::new())),
            messages: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn connect(&self, name: &'a str, url: &'a str) {
        let (tx, _) = mpsc::channel();

        self.connections.lock().unwrap().insert(name, tx);
        self.messages.lock().unwrap().insert(name, Vec::new());

        match Url::parse(url) {
            Ok(url) => {
                tokio::spawn(async move {
                    let (mut stream, _) = connect_async(url.to_string())
                        .await
                        .unwrap_or_else(|_| panic!("failed connecting to {}", url));

                    while let Some(message) = stream.next().await {
                        if let Ok(_message) = message {
                            // TODO: Decode flexbuffer message
                            // error!("breaks thread so...")
                        }
                    }
                });
            }
            Err(err) => {
                error!("{:?}", err);
            }
        }
    }
}
