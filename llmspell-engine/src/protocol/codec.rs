//! Codec for LRP/LDP protocol message framing
//!
//! Uses length-delimited framing with JSON serialization

use bytes::BytesMut;
use std::io;
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};

use crate::protocol::message::ProtocolMessage;

/// Codec for LRP/LDP protocol messages
///
/// Uses 4-byte big-endian length prefix followed by JSON payload
#[derive(Debug, Clone)]
pub struct LRPCodec {
    inner: LengthDelimitedCodec,
}

impl LRPCodec {
    /// Create a new LRP codec with default settings
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: LengthDelimitedCodec::builder()
                .length_field_length(4)
                .big_endian()
                .max_frame_length(16 * 1024 * 1024) // 16MB max message size
                .new_codec(),
        }
    }
}

impl Default for LRPCodec {
    fn default() -> Self {
        Self::new()
    }
}

impl Decoder for LRPCodec {
    type Item = ProtocolMessage;
    type Error = io::Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Use inner codec to get the frame
        match self.inner.decode(src)? {
            Some(frame) => {
                // Deserialize JSON from the frame
                let msg: ProtocolMessage = serde_json::from_slice(&frame)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Ok(Some(msg))
            }
            None => Ok(None),
        }
    }
}

impl Encoder<ProtocolMessage> for LRPCodec {
    type Error = io::Error;

    fn encode(&mut self, item: ProtocolMessage, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // Serialize message to JSON
        let json =
            serde_json::to_vec(&item).map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

        // Use inner codec to add length framing
        self.inner.encode(json.into(), dst)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::message::{MessageType, ProtocolMessage};

    #[test]
    fn test_codec_round_trip() {
        let mut codec = LRPCodec::new();
        let mut buf = BytesMut::new();

        // Create a test message
        let msg = ProtocolMessage {
            msg_id: "test-123".to_string(),
            msg_type: MessageType::Request,
            channel: "shell".to_string(),
            content: serde_json::json!({"test": "data"}),
        };

        // Encode
        codec.encode(msg.clone(), &mut buf).unwrap();

        // Decode
        let decoded = codec.decode(&mut buf).unwrap();
        assert!(decoded.is_some());

        let decoded_msg = decoded.unwrap();
        assert_eq!(decoded_msg.msg_id, msg.msg_id);
        assert_eq!(decoded_msg.content, msg.content);
    }
}
