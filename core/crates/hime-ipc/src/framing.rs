use std::fmt;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

pub const MAGIC: [u8; 4] = [b'H', b'I', b'M', b'E'];
pub const VERSION: u8 = 0x01;
pub const HEADER_SIZE: usize = 12;
pub const MAX_PAYLOAD_SIZE: usize = 16 * 1024 * 1024; // 16 MiB

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
    type Error = FramingError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(MsgType::Request),
            0x02 => Ok(MsgType::Response),
            0x03 => Ok(MsgType::Notification),
            0x04 => Ok(MsgType::StreamChunk),
            0x05 => Ok(MsgType::StreamEnd),
            0x06 => Ok(MsgType::Cancel),
            other => Err(FramingError::InvalidMessageType(other)),
        }
    }
}

impl fmt::Display for MsgType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MsgType::Request => write!(f, "Request(0x01)"),
            MsgType::Response => write!(f, "Response(0x02)"),
            MsgType::Notification => write!(f, "Notification(0x03)"),
            MsgType::StreamChunk => write!(f, "StreamChunk(0x04)"),
            MsgType::StreamEnd => write!(f, "StreamEnd(0x05)"),
            MsgType::Cancel => write!(f, "Cancel(0x06)"),
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
    pub fn new(msg_type: MsgType, payload_len: u32) -> Result<Self, FramingError> {
        if payload_len as usize > MAX_PAYLOAD_SIZE {
            return Err(FramingError::PayloadTooLarge {
                size: payload_len as usize,
                max: MAX_PAYLOAD_SIZE,
            });
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

    pub fn parse(buf: &[u8; HEADER_SIZE]) -> Result<Self, FramingError> {
        let mut magic = [0u8; 4];
        magic.copy_from_slice(&buf[0..4]);
        if magic != MAGIC {
            return Err(FramingError::InvalidMagic(magic));
        }

        let version = buf[4];
        if version != VERSION {
            return Err(FramingError::UnsupportedVersion(version));
        }

        let msg_type = MsgType::try_from(buf[5])?;

        let reserved = u16::from_be_bytes([buf[6], buf[7]]);
        if reserved != 0 {
            return Err(FramingError::NonZeroReserved(reserved));
        }

        let payload_len = u32::from_be_bytes([buf[8], buf[9], buf[10], buf[11]]);
        if payload_len as usize > MAX_PAYLOAD_SIZE {
            return Err(FramingError::PayloadTooLarge {
                size: payload_len as usize,
                max: MAX_PAYLOAD_SIZE,
            });
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
    pub fn new(msg_type: MsgType, payload: Vec<u8>) -> Result<Self, FramingError> {
        let header = FrameHeader::new(msg_type, payload.len() as u32)?;
        Ok(Self { header, payload })
    }

    pub async fn read_from<R: AsyncRead + Unpin>(reader: &mut R) -> Result<Self, FramingError> {
        let mut header_buf = [0u8; HEADER_SIZE];
        reader.read_exact(&mut header_buf).await.map_err(|e| {
            if e.kind() == std::io::ErrorKind::UnexpectedEof {
                FramingError::ConnectionClosed
            } else {
                FramingError::Io(e)
            }
        })?;

        let header = FrameHeader::parse(&header_buf)?;

        // Pre-allocation check already performed inside FrameHeader::parse
        let mut payload = vec![0u8; header.payload_len as usize];
        if header.payload_len > 0 {
            reader.read_exact(&mut payload).await.map_err(|e| {
                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                    FramingError::UnexpectedEofInPayload
                } else {
                    FramingError::Io(e)
                }
            })?;
        }

        Ok(Self { header, payload })
    }

    pub async fn write_to<W: AsyncWrite + Unpin>(
        &self,
        writer: &mut W,
    ) -> Result<(), FramingError> {
        let header_bytes = self.header.to_bytes();
        writer
            .write_all(&header_bytes)
            .await
            .map_err(FramingError::Io)?;
        if !self.payload.is_empty() {
            writer
                .write_all(&self.payload)
                .await
                .map_err(FramingError::Io)?;
        }
        writer.flush().await.map_err(FramingError::Io)?;
        Ok(())
    }
}

#[derive(Error, Debug)]
pub enum FramingError {
    #[error("Invalid magic identifier: {0:?}, expected b\"HIME\"")]
    InvalidMagic([u8; 4]),

    #[error("Unsupported protocol version: {0}, expected 0x01")]
    UnsupportedVersion(u8),

    #[error("Invalid message type: {0:#04x}")]
    InvalidMessageType(u8),

    #[error("Reserved field must be 0x0000, got: {0:#06x}")]
    NonZeroReserved(u16),

    #[error("Payload length {size} exceeds maximum limit of {max} bytes (16 MiB)")]
    PayloadTooLarge { size: usize, max: usize },

    #[error("Connection closed cleanly by peer")]
    ConnectionClosed,

    #[error("Stream truncated while reading frame payload")]
    UnexpectedEofInPayload,

    #[error("I/O transport error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_valid_header_encoding_decoding() {
        let header = FrameHeader::new(MsgType::Request, 75).unwrap();
        let bytes = header.to_bytes();
        assert_eq!(&bytes[0..4], b"HIME");
        assert_eq!(bytes[4], 0x01);
        assert_eq!(bytes[5], 0x01);
        assert_eq!(&bytes[6..8], &[0x00, 0x00]);
        assert_eq!(&bytes[8..12], &[0x00, 0x00, 0x00, 0x4B]);

        let parsed = FrameHeader::parse(&bytes).unwrap();
        assert_eq!(header, parsed);
    }

    #[test]
    fn test_test_vector_1_handshake_request() {
        let mut raw = [0u8; 12];
        raw[0..4].copy_from_slice(b"HIME");
        raw[4] = 0x01;
        raw[5] = 0x01; // Request
        raw[6] = 0x00;
        raw[7] = 0x00;
        raw[8..12].copy_from_slice(&75u32.to_be_bytes());

        let header = FrameHeader::parse(&raw).unwrap();
        assert_eq!(header.msg_type, MsgType::Request);
        assert_eq!(header.payload_len, 75);
    }

    #[test]
    fn test_test_vector_2_stream_chunk() {
        let mut raw = [0u8; 12];
        raw[0..4].copy_from_slice(b"HIME");
        raw[4] = 0x01;
        raw[5] = 0x04; // StreamChunk
        raw[6] = 0x00;
        raw[7] = 0x00;
        raw[8..12].copy_from_slice(&80u32.to_be_bytes());

        let header = FrameHeader::parse(&raw).unwrap();
        assert_eq!(header.msg_type, MsgType::StreamChunk);
        assert_eq!(header.payload_len, 80);
    }

    #[test]
    fn test_reject_wrong_magic() {
        let mut raw = [0u8; 12];
        raw[0..4].copy_from_slice(b"NOPE");
        assert!(matches!(
            FrameHeader::parse(&raw),
            Err(FramingError::InvalidMagic(_))
        ));
    }

    #[test]
    fn test_reject_wrong_version() {
        let mut raw = [0u8; 12];
        raw[0..4].copy_from_slice(b"HIME");
        raw[4] = 0x02; // Version 2
        assert!(matches!(
            FrameHeader::parse(&raw),
            Err(FramingError::UnsupportedVersion(2))
        ));
    }

    #[test]
    fn test_reject_reserved_nonzero() {
        let mut raw = [0u8; 12];
        raw[0..4].copy_from_slice(b"HIME");
        raw[4] = 0x01;
        raw[5] = 0x01;
        raw[6] = 0x01; // Reserved non-zero
        assert!(matches!(
            FrameHeader::parse(&raw),
            Err(FramingError::NonZeroReserved(_))
        ));
    }

    #[test]
    fn test_preallocation_check_rejects_oversized_payload() {
        let mut raw = [0u8; 12];
        raw[0..4].copy_from_slice(b"HIME");
        raw[4] = 0x01;
        raw[5] = 0x01;
        let oversized = (MAX_PAYLOAD_SIZE as u32) + 1;
        raw[8..12].copy_from_slice(&oversized.to_be_bytes());

        assert!(matches!(
            FrameHeader::parse(&raw),
            Err(FramingError::PayloadTooLarge { .. })
        ));
    }

    #[tokio::test]
    async fn test_frame_roundtrip_async() {
        let payload = b"Hello Hime IPC System!".to_vec();
        let frame = Frame::new(MsgType::Request, payload.clone()).unwrap();

        let mut stream = Cursor::new(Vec::new());
        frame.write_to(&mut stream).await.unwrap();

        stream.set_position(0);
        let decoded = Frame::read_from(&mut stream).await.unwrap();
        assert_eq!(decoded.header.msg_type, MsgType::Request);
        assert_eq!(decoded.payload, payload);
    }
}
