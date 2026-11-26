//! Hime Named Pipe Framing and Client Protocol Implementation
//!
//! Implements the 12-byte binary framing protocol and JSON-RPC 2.0 communication
//! between the Hime (Bloom) Desktop Shell and the standalone Hime Core daemon.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::sync::atomic::{AtomicU64, Ordering};

pub const MAGIC: [u8; 4] = *b"HIME";
pub const VERSION: u8 = 0x01;
pub const HEADER_SIZE: usize = 12;
pub const MAX_PAYLOAD_SIZE: usize = 16 * 1024 * 1024; // 16 MiB
pub const DEFAULT_HIME_PIPE: &str = r"\\.\pipe\hime-ipc";

static REQUEST_COUNTER: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MsgType {
    Request = 0x01,
    Response = 0x02,
    Notification = 0x03,
    StreamChunk = 0x04,
    StreamEnd = 0x05,
    Cancel = 0x06,
}

impl TryFrom<u8> for MsgType {
    type Error = String;

    fn try_from(val: u8) -> Result<Self, Self::Error> {
        match val {
            0x01 => Ok(MsgType::Request),
            0x02 => Ok(MsgType::Response),
            0x03 => Ok(MsgType::Notification),
            0x04 => Ok(MsgType::StreamChunk),
            0x05 => Ok(MsgType::StreamEnd),
            0x06 => Ok(MsgType::Cancel),
            other => Err(format!("Invalid message type byte: 0x{:02X}", other)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrameHeader {
    pub magic: [u8; 4],
    pub version: u8,
    pub msg_type: MsgType,
    pub reserved: u16,
    pub payload_len: u32,
}

impl FrameHeader {
    pub fn new(msg_type: MsgType, payload_len: u32) -> Result<Self, String> {
        if payload_len as usize > MAX_PAYLOAD_SIZE {
            return Err(format!(
                "Payload size {} exceeds maximum {}",
                payload_len, MAX_PAYLOAD_SIZE
            ));
        }
        Ok(Self {
            magic: MAGIC,
            version: VERSION,
            msg_type,
            reserved: 0,
            payload_len,
        })
    }

    pub fn to_bytes(&self) -> [u8; HEADER_SIZE] {
        let mut buf = [0u8; HEADER_SIZE];
        buf[0..4].copy_from_slice(&self.magic);
        buf[4] = self.version;
        buf[5] = self.msg_type as u8;
        buf[6..8].copy_from_slice(&self.reserved.to_be_bytes());
        buf[8..12].copy_from_slice(&self.payload_len.to_be_bytes());
        buf
    }

    pub fn parse(buf: &[u8; HEADER_SIZE]) -> Result<Self, String> {
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&buf[0..4]);
        if magic != MAGIC {
            return Err(format!("Invalid magic header: {:?}", magic));
        }

        let version = buf[4];
        if version != VERSION {
            return Err(format!("Unsupported framing version: {}", version));
        }

        let msg_type = MsgType::try_from(buf[5])?;
        let reserved = u16::from_be_bytes([buf[6], buf[7]]);
        if reserved != 0 {
            return Err(format!("Non-zero reserved field: {}", reserved));
        }

        let payload_len = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        if payload_len as usize > MAX_PAYLOAD_SIZE {
            return Err(format!("Payload too large: {} bytes", payload_len));
        }

        Ok(Self {
            magic,
            version,
            msg_type,
            reserved,
            payload_len,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frame {
    pub header: FrameHeader,
    pub payload: Vec<u8>,
}

impl Frame {
    pub fn new(msg_type: MsgType, payload: Vec<u8>) -> Result<Self, String> {
        let header = FrameHeader::new(msg_type, payload.len() as u32)?;
        Ok(Self { header, payload })
    }

    pub fn write_to<W: Write>(&self, writer: &mut W) -> Result<(), String> {
        writer
            .write_all(&self.header.to_bytes())
            .map_err(|e| format!("Failed to write header: {}", e))?;
        if !self.payload.is_empty() {
            writer
                .write_all(&self.payload)
                .map_err(|e| format!("Failed to write payload: {}", e))?;
        }
        writer.flush().map_err(|e| format!("Flush failed: {}", e))?;
        Ok(())
    }

    pub fn read_from<R: Read>(reader: &mut R) -> Result<Self, String> {
        let mut header_buf = [0u8; HEADER_SIZE];
        reader
            .read_exact(&mut header_buf)
            .map_err(|e| format!("Failed to read frame header: {}", e))?;

        let header = FrameHeader::parse(&header_buf)?;
        let mut payload = vec![0u8; header.payload_len as usize];
        if header.payload_len > 0 {
            reader
                .read_exact(&mut payload)
                .map_err(|e| format!("Failed to read frame payload: {}", e))?;
        }

        Ok(Self { header, payload })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub id: Value,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<Value>,
}

impl JsonRpcRequest {
    pub fn new(id: impl Into<Value>, method: &str, params: Option<Value>) -> Self {
        Self {
            jsonrpc: "2.0".to_string(),
            id: id.into(),
            method: method.to_string(),
            params,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    pub id: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JsonRpcError {
    pub code: i64,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Connects to the Windows Named Pipe of Hime Core.
pub fn connect_named_pipe(pipe_path: Option<&str>) -> std::io::Result<File> {
    let path = pipe_path.unwrap_or(DEFAULT_HIME_PIPE);
    OpenOptions::new().read(true).write(true).open(path)
}

/// Executes a synchronous JSON-RPC request over an active Named Pipe stream.
pub fn rpc_call(file: &mut File, method: &str, params: Option<Value>) -> Result<Value, String> {
    let req_id = REQUEST_COUNTER.fetch_add(1, Ordering::SeqCst);
    let rpc_req = JsonRpcRequest::new(req_id, method, params);
    let payload =
        serde_json::to_vec(&rpc_req).map_err(|e| format!("Failed to serialize request: {}", e))?;

    let frame = Frame::new(MsgType::Request, payload)?;
    frame.write_to(file)?;

    let resp_frame = Frame::read_from(file)?;
    if resp_frame.header.msg_type != MsgType::Response {
        return Err(format!(
            "Expected response frame, got {:?}",
            resp_frame.header.msg_type
        ));
    }

    let rpc_resp: JsonRpcResponse = serde_json::from_slice(&resp_frame.payload)
        .map_err(|e| format!("Failed to parse JSON-RPC response: {}", e))?;

    if let Some(err) = rpc_resp.error {
        return Err(format!("RPC error {}: {}", err.code, err.message));
    }

    rpc_resp
        .result
        .ok_or_else(|| "Empty result in RPC response".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_header_roundtrip() {
        let header = FrameHeader::new(MsgType::Request, 42).unwrap();
        let bytes = header.to_bytes();
        let parsed = FrameHeader::parse(&bytes).unwrap();
        assert_eq!(header, parsed);
        assert_eq!(parsed.payload_len, 42);
        assert_eq!(parsed.msg_type, MsgType::Request);
    }

    #[test]
    fn test_reject_invalid_magic() {
        let mut bytes = FrameHeader::new(MsgType::Request, 10).unwrap().to_bytes();
        bytes[0] = b'X';
        assert!(FrameHeader::parse(&bytes).is_err());
    }

    #[test]
    fn test_reject_unsupported_version() {
        let mut bytes = FrameHeader::new(MsgType::Request, 10).unwrap().to_bytes();
        bytes[4] = 0x99;
        assert!(FrameHeader::parse(&bytes).is_err());
    }

    #[test]
    fn test_frame_io_roundtrip() {
        let frame = Frame::new(MsgType::Response, b"{\"status\":\"pong\"}".to_vec()).unwrap();
        let mut buffer = Vec::new();
        frame.write_to(&mut buffer).unwrap();

        let mut cursor = std::io::Cursor::new(buffer);
        let read_frame = Frame::read_from(&mut cursor).unwrap();
        assert_eq!(frame, read_frame);
    }
}
