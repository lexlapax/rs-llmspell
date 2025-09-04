//! Heartbeat channel handler for Jupyter protocol
//!
//! The heartbeat channel is a simple echo service that responds to ping messages
//! from Jupyter clients to verify the kernel is alive.

use anyhow::{Context, Result};
use tokio::time::{interval, Duration};
use zmq::{Context as ZmqContext, Socket, SocketType};

/// Dedicated heartbeat handler for kernel liveness detection
pub struct HeartbeatHandler {
    socket: Socket,
    is_running: bool,
}

impl HeartbeatHandler {
    /// Create and bind heartbeat socket
    ///
    /// # Errors
    ///
    /// Returns an error if socket creation or binding fails.
    pub fn bind(context: &ZmqContext, addr: &str) -> Result<Self> {
        let socket = context
            .socket(SocketType::REP)
            .context("Failed to create heartbeat socket")?;

        socket
            .bind(addr)
            .with_context(|| format!("Failed to bind heartbeat socket to {addr}"))?;

        // Set non-blocking mode
        socket
            .set_rcvtimeo(100)
            .context("Failed to set heartbeat recv timeout")?;

        Ok(Self {
            socket,
            is_running: false,
        })
    }

    /// Start heartbeat echo service
    ///
    /// # Errors
    ///
    /// Returns an error if the heartbeat service encounters a fatal error.
    pub async fn start(&mut self) -> Result<()> {
        self.is_running = true;
        let mut heartbeat_interval = interval(Duration::from_millis(100));

        tracing::debug!("Starting heartbeat handler");

        if self.is_running {
            loop {
                heartbeat_interval.tick().await;

                if let Err(e) = self.handle_heartbeat() {
                    tracing::warn!("Heartbeat error: {}", e);
                    // Continue running on heartbeat errors
                }

                if !self.is_running {
                    break;
                }
            }
        }

        tracing::debug!("Heartbeat handler stopped");
        Ok(())
    }

    /// Stop heartbeat service
    pub const fn stop(&mut self) {
        self.is_running = false;
    }

    /// Handle single heartbeat request (echo back immediately)
    fn handle_heartbeat(&self) -> Result<()> {
        match self.socket.recv_bytes(zmq::DONTWAIT) {
            Ok(data) => {
                // Echo the data back immediately
                self.socket
                    .send(&data, 0)
                    .context("Failed to send heartbeat response")?;
                tracing::trace!("Heartbeat echo: {} bytes", data.len());
            }
            Err(zmq::Error::EAGAIN) => {
                // No heartbeat received, this is normal
            }
            Err(e) => {
                return Err(e).context("Heartbeat receive error");
            }
        }
        Ok(())
    }
}

impl Drop for HeartbeatHandler {
    fn drop(&mut self) {
        self.stop();
        tracing::debug!("HeartbeatHandler dropped");
    }
}
