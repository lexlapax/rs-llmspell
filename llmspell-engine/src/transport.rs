//! Transport abstraction for protocol messages
//!
//! Provides a trait for different transport implementations (TCP, WebSocket, etc.)

use async_trait::async_trait;
use std::fmt::Debug;
use thiserror::Error;

use crate::protocol::message::ProtocolMessage;

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

/// Mock transport for testing
pub mod mock {
    use super::{async_trait, Debug, ProtocolMessage, Transport, TransportError};
    use std::collections::VecDeque;
    use tokio::sync::Mutex;

    /// Mock transport for testing
    #[derive(Debug)]
    pub struct MockTransport {
        send_queue: Mutex<VecDeque<ProtocolMessage>>,
        recv_queue: Mutex<VecDeque<ProtocolMessage>>,
        connected: Mutex<bool>,
    }

    impl Default for MockTransport {
        fn default() -> Self {
            Self::new()
        }
    }

    impl MockTransport {
        /// Create a new mock transport
        #[must_use]
        pub fn new() -> Self {
            Self {
                send_queue: Mutex::new(VecDeque::new()),
                recv_queue: Mutex::new(VecDeque::new()),
                connected: Mutex::new(true),
            }
        }

        /// Add a message to the receive queue for testing
        #[allow(dead_code)]
        pub async fn add_recv_message(&self, msg: ProtocolMessage) {
            self.recv_queue.lock().await.push_back(msg);
        }

        /// Get sent messages for verification
        #[allow(dead_code)]
        pub async fn get_sent_messages(&self) -> Vec<ProtocolMessage> {
            self.send_queue.lock().await.drain(..).collect()
        }
    }

    #[async_trait]
    impl Transport for MockTransport {
        async fn send(&mut self, msg: ProtocolMessage) -> Result<(), TransportError> {
            if !*self.connected.lock().await {
                return Err(TransportError::ConnectionClosed);
            }
            self.send_queue.lock().await.push_back(msg);
            Ok(())
        }

        async fn recv(&mut self) -> Result<ProtocolMessage, TransportError> {
            if !*self.connected.lock().await {
                return Err(TransportError::ConnectionClosed);
            }
            self.recv_queue
                .lock()
                .await
                .pop_front()
                .ok_or(TransportError::ConnectionClosed)
        }

        async fn close(&mut self) -> Result<(), TransportError> {
            *self.connected.lock().await = false;
            Ok(())
        }

        fn is_connected(&self) -> bool {
            // Can't use async in non-async function, so return true for simplicity
            true
        }
    }
}

/// TCP transport implementation
pub mod tcp {
    use super::{async_trait, Debug, ProtocolMessage, Transport, TransportError};
    use crate::protocol::codec::LRPCodec;
    use futures::{SinkExt, StreamExt};
    use tokio::net::TcpStream;
    use tokio_util::codec::Framed;

    use futures::stream::{SplitSink, SplitStream};
    use std::sync::Arc;
    use tokio::sync::Mutex as TokioMutex;

    /// Type alias for the send half of a framed TCP connection
    type TcpSink = Arc<TokioMutex<SplitSink<Framed<TcpStream, LRPCodec>, ProtocolMessage>>>;

    /// Type alias for the receive half of a framed TCP connection  
    type TcpRecvStream = Arc<TokioMutex<SplitStream<Framed<TcpStream, LRPCodec>>>>;

    /// TCP transport using framed codec with split send/receive
    #[derive(Debug)]
    pub struct TcpTransport {
        sink: Option<TcpSink>,
        stream: Option<TcpRecvStream>,
    }

    impl TcpTransport {
        /// Create a new TCP transport from a connected stream
        pub fn new(stream: TcpStream) -> Self {
            let framed = Framed::new(stream, LRPCodec::new());
            let (sink, stream) = framed.split();
            Self {
                sink: Some(Arc::new(TokioMutex::new(sink))),
                stream: Some(Arc::new(TokioMutex::new(stream))),
            }
        }

        /// Connect to a TCP address
        ///
        /// # Errors
        ///
        /// Returns `TransportError::Io` if connection fails
        pub async fn connect(addr: &str) -> Result<Self, TransportError> {
            let stream = TcpStream::connect(addr).await?;
            Ok(Self::new(stream))
        }
    }

    #[async_trait]
    impl Transport for TcpTransport {
        async fn send(&mut self, msg: ProtocolMessage) -> Result<(), TransportError> {
            if let Some(sink_ref) = &self.sink {
                let mut sink = sink_ref.lock().await;
                sink.send(msg)
                    .await
                    .map_err(|e| TransportError::Io(std::io::Error::other(e.to_string())))
            } else {
                Err(TransportError::ConnectionClosed)
            }
        }

        async fn recv(&mut self) -> Result<ProtocolMessage, TransportError> {
            if let Some(stream_ref) = &self.stream {
                let mut stream = stream_ref.lock().await;
                match stream.next().await {
                    Some(Ok(msg)) => Ok(msg),
                    Some(Err(e)) => Err(TransportError::Io(std::io::Error::other(e.to_string()))),
                    None => Err(TransportError::ConnectionClosed),
                }
            } else {
                Err(TransportError::ConnectionClosed)
            }
        }

        async fn close(&mut self) -> Result<(), TransportError> {
            self.sink = None;
            self.stream = None;
            Ok(())
        }

        fn is_connected(&self) -> bool {
            self.sink.is_some() && self.stream.is_some()
        }
    }
}
