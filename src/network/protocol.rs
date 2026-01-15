use serde::{Deserialize, Serialize};
use crate::io::FileInfo;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Message {
    FileInfo(Vec<FileInfo>),
    // 추후 파일 데이터 청크 전송 등을 위한 메시지 확장 가능
    // FileChunk(usize, Vec<u8>), 
}
