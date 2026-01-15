mod protocol;
mod traits;

#[cfg(not(target_arch = "wasm32"))]
mod native;
#[cfg(target_arch = "wasm32")]
mod wasm;

use egui::Ui;
use anyhow::Result;
use crate::io::FileInfo;

pub use protocol::Message;
pub use traits::{FileSender, FileReceiver};

#[cfg(not(target_arch = "wasm32"))]
use native::{NativeSender as SenderImpl, NativeReceiver as ReceiverImpl};
#[cfg(target_arch = "wasm32")]
use wasm::{WasmSender as SenderImpl, WasmReceiver as ReceiverImpl};

pub struct FileSenderUi {
    addr_str: String,
    sender: Box<dyn FileSender>,
}

impl FileSenderUi {
    pub fn new() -> Self {
        Self { addr_str: "127.0.0.1:47102".to_string(), sender: Box::new(SenderImpl) }
    }
    pub fn ui(&mut self, ui: &mut Ui, files: Vec<FileInfo>) -> Result<()> {
        ui.heading("Sender (Server)");
        ui.text_edit_singleline(&mut self.addr_str);
        if ui.button("Start Listening").clicked() {
            self.sender.start(&self.addr_str, files)?;
        }
        Ok(())
    }
}

impl Default for FileSenderUi {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for FileDownloaderUi {
    fn default() -> Self {
        Self::new()
    }
}

pub struct FileDownloaderUi {
    addr_str: String,
    receiver: Box<dyn FileReceiver>,
}

impl FileDownloaderUi {
    pub fn new() -> Self {
        Self { addr_str: "127.0.0.1:47102".to_string(), receiver: Box::new(ReceiverImpl::new()) }
    }
    pub fn ui(&mut self, ui: &mut Ui) -> Result<()> {
        ui.heading("Receiver (Client)");
        ui.text_edit_singleline(&mut self.addr_str);
        if !self.receiver.is_connected() {
            if ui.button("Connect to Server").clicked() {
                self.receiver.connect(&self.addr_str)?;
            }
        } else {
            ui.label("Connected!");
        }
        self.receiver.poll()?;
        Ok(())
    }
}