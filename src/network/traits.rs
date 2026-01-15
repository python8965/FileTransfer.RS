use anyhow::Result;
use crate::io::FileInfo;

/// 서버 역할을 하며 파일을 보내는 인터페이스
pub trait FileSender {
    fn start(&mut self, addr: &str, files: Vec<FileInfo>) -> Result<()>;
}

/// 클라이언트 역할을 하며 파일을 받는 인터페이스
pub trait FileReceiver {
    fn connect(&mut self, addr: &str) -> Result<()>;
    fn poll(&mut self) -> Result<()>;
    fn is_connected(&self) -> bool;
    fn last_error(&self) -> Option<String>;
}