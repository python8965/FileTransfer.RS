use anyhow::{Result, anyhow};
use ewebsock::{WsMessage, WsReceiver, WsSender, WsEvent};
use tracing::{info, error};
use crate::io::FileInfo;
use super::traits::{FileSender, FileReceiver};
use super::protocol::Message;

pub struct WasmSender;
impl FileSender for WasmSender {
    fn start(&mut self, _addr: &str, _files: Vec<FileInfo>) -> Result<()> {
        Err(anyhow!("WASM cannot act as a server listener"))
    }
}

pub struct WasmReceiver {
    sender: Option<WsSender>,
    receiver: Option<WsReceiver>,
    connected: bool,
    last_err: Option<String>,
}

impl WasmReceiver {
    pub fn new() -> Self {
        Self { sender: None, receiver: None, connected: false, last_err: None }
    }
}

impl FileReceiver for WasmReceiver {
    fn connect(&mut self, addr: &str) -> Result<()> {
        let url = if addr.starts_with("ws://") { addr.to_string() } else { format!("ws://{}", addr) };
        let (sender, receiver) = ewebsock::connect(url, ewebsock::Options::default()).map_err(|e| anyhow!(e))?;
        self.sender = Some(sender);
        self.receiver = Some(receiver);
        Ok(())
    }

    fn poll(&mut self) -> Result<()> {
        if let Some(rx) = &mut self.receiver {
            while let Some(event) = rx.try_recv() {
                match event {
                    WsEvent::Opened => self.connected = true,
                    WsEvent::Message(WsMessage::Binary(data)) => {
                        let msg: Message = rmp_serde::from_slice(&data)?;
                        info!("WASM Received: {:?}", msg);
                    }
                    WsEvent::Error(e) => self.last_err = Some(e),
                    WsEvent::Closed => self.connected = false,
                    _ => {}
                }
            }
        }
        Ok(())
    }

    fn is_connected(&self) -> bool { self.connected }
    fn last_error(&self) -> Option<String> { self.last_err.clone() }
}