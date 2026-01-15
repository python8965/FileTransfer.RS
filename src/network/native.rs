use std::net::{TcpListener};
use std::thread;
use anyhow::{Result};
use tungstenite::{accept, connect as ws_connect, Message as WsMessage};
use tracing::{info, error, debug};

use crate::io::{FileInfo};
use super::protocol::Message;
use super::traits::{FileSender, FileReceiver};

pub struct NativeSender;

impl FileSender for NativeSender {
    fn start(&mut self, addr: &str, files: Vec<FileInfo>) -> Result<()> {
        let addr = addr.to_string();
        thread::spawn(move || {
            if let Err(e) = run_server(&addr, files) {
                error!("Server error: {:?}", e);
            }
        });
        Ok(())
    }
}

fn run_server(addr: &str, files: Vec<FileInfo>) -> Result<()> {
    let listener = TcpListener::bind(addr)?;
    info!("Server listening on {}", addr);
    
    let (stream, _) = listener.accept()?;
    let mut websocket = accept(stream)?;
    
    // 파일 정보 목록 전송 (MessagePack)
    let msg_data = rmp_serde::to_vec(&Message::FileInfo(files.clone()))?;
    websocket.send(WsMessage::Binary(msg_data.into()))?;
    
    // TODO: 파일 본문 전송 로직 구현
    info!("Sent file list to client");
    Ok(())
}

pub struct NativeReceiver {
    connected: bool,
    last_err: Option<String>,
}

impl NativeReceiver {
    pub fn new() -> Self {
        Self { connected: false, last_err: None }
    }
}

impl FileReceiver for NativeReceiver {
    fn connect(&mut self, addr: &str) -> Result<()> {
        let url = format!("ws://{}", addr);
        let (mut socket, _) = ws_connect(url)?;
        self.connected = true;
        
        thread::spawn(move || {
            loop {
                match socket.read() {
                    Ok(WsMessage::Binary(data)) => {
                        let msg: Message = rmp_serde::from_slice(&data).unwrap();
                        debug!("Received: {:?}", msg);
                    }
                    Ok(WsMessage::Close(_)) => break,
                    Err(e) => {
                        error!("Read error: {:?}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });
        Ok(())
    }

    fn poll(&mut self) -> Result<()> { Ok(()) }
    fn is_connected(&self) -> bool { self.connected }
    fn last_error(&self) -> Option<String> { self.last_err.clone() }
}