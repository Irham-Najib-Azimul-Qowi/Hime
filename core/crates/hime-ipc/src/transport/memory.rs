use tokio::io::{duplex, DuplexStream};

/// Creates a pair of connected in-memory duplex streams for deterministic testing.
pub fn create_memory_duplex(max_buf_size: usize) -> (DuplexStream, DuplexStream) {
    duplex(max_buf_size)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::framing::{Frame, MsgType};

    #[tokio::test]
    async fn test_duplex_framing_roundtrip() {
        let (mut client, mut server) = create_memory_duplex(1024 * 1024);

        let send_payload = b"duplex-test-payload".to_vec();
        let send_frame = Frame::new(MsgType::Request, send_payload.clone()).unwrap();

        tokio::spawn(async move {
            send_frame.write_to(&mut client).await.unwrap();
        });

        let received = Frame::read_from(&mut server).await.unwrap();
        assert_eq!(received.payload, send_payload);
    }
}
