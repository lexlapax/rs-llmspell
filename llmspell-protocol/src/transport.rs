//! Transport abstraction for protocol messages
//!
//! Provides a trait for different transport implementations (TCP, WebSocket, etc.)

use async_trait::async_trait;
use std::fmt::Debug;
use thiserror::Error;

use crate::message::ProtocolMessage;

/// Errors that can occur during transport operations
#[derive(Error, Debug)]
pub enum TransportError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Connection closed")]
    ConnectionClosed,

    #[error("Protocol error: {0}")]
    Protocol(String),
}

/// Transport trait for sending and receiving protocol messages
#[async_trait]
pub trait Transport: Send + Sync + Debug {
    /// Send a protocol message
    async fn send(&mut self, msg: ProtocolMessage) -> Result<(), TransportError>;

    /// Receive a protocol message
    async fn recv(&mut self) -> Result<ProtocolMessage, TransportError>;

    /// Close the transport connection
    async fn close(&mut self) -> Result<(), TransportError>;

    /// Check if the transport is connected
    fn is_connected(&self) -> bool;
}

/// TCP transport implementation
pub mod tcp {
    use super::*;
    use crate::codec::LRPCodec;
    use futures::{SinkExt, StreamExt};
    use tokio::net::TcpStream;
    use tokio_util::codec::Framed;

    /// TCP transport using framed codec
    #[derive(Debug)]
    pub struct TcpTransport {
        stream: Option<Framed<TcpStream, LRPCodec>>,
    }

    impl TcpTransport {
        /// Create a new TCP transport from a connected stream
        pub fn new(stream: TcpStream) -> Self {
            let framed = Framed::new(stream, LRPCodec::new());
            Self {
                stream: Some(framed),
            }
        }

        /// Connect to a TCP address
        pub async fn connect(addr: &str) -> Result<Self, TransportError> {
            let stream = TcpStream::connect(addr).await?;
            Ok(Self::new(stream))
        }
    }

    #[async_trait]
    impl Transport for TcpTransport {
        async fn send(&mut self, msg: ProtocolMessage) -> Result<(), TransportError> {
            if let Some(stream) = &mut self.stream {
                stream.send(msg).await.map_err(|e| {
                    TransportError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ))
                })
            } else {
                Err(TransportError::ConnectionClosed)
            }
        }

        async fn recv(&mut self) -> Result<ProtocolMessage, TransportError> {
            if let Some(stream) = &mut self.stream {
                match stream.next().await {
                    Some(Ok(msg)) => Ok(msg),
                    Some(Err(e)) => Err(TransportError::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ))),
                    None => Err(TransportError::ConnectionClosed),
                }
            } else {
                Err(TransportError::ConnectionClosed)
            }
        }

        async fn close(&mut self) -> Result<(), TransportError> {
            self.stream = None;
            Ok(())
        }

        fn is_connected(&self) -> bool {
            self.stream.is_some()
        }
    }
}
