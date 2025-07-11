// ABOUTME: Connection pooling abstraction for various resources (HTTP clients, database connections)
// ABOUTME: Provides a generic pooling mechanism with health checks and lifecycle management

use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, Instant};
use thiserror::Error;
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, info, warn};

/// Connection pool error types
#[derive(Debug, Error)]
pub enum PoolError {
    #[error("Pool exhausted: no connections available")]
    /// Pool has no available connections
    Exhausted,

    #[error("Connection creation failed: {message}")]
    /// Failed to create a new connection
    CreationFailed {
        /// Error message describing the failure
        message: String,
    },

    #[error("Connection validation failed: {message}")]
    /// Connection validation failed
    ValidationFailed {
        /// Error message describing the validation failure
        message: String,
    },

    #[error("Pool is shutting down")]
    /// Pool is in the process of shutting down
    ShuttingDown,

    #[error("Invalid pool configuration: {message}")]
    /// Pool configuration is invalid
    InvalidConfiguration {
        /// Error message describing the configuration issue
        message: String,
    },
}

/// Trait for poolable connections
#[async_trait::async_trait]
pub trait PoolableConnection: Send + Sync + 'static {
    /// The error type for this connection
    type Error: std::error::Error + Send + Sync + 'static;

    /// Check if the connection is still valid
    async fn is_valid(&self) -> bool;

    /// Perform any cleanup when returning to pool
    async fn reset(&mut self) -> Result<(), Self::Error>;

    /// Close the connection
    async fn close(self) -> Result<(), Self::Error>;
}

/// Trait for creating connections
#[async_trait::async_trait]
pub trait ConnectionFactory: Send + Sync + 'static {
    /// The connection type this factory creates
    type Connection: PoolableConnection;

    /// Create a new connection
    async fn create(
        &self,
    ) -> Result<Self::Connection, <Self::Connection as PoolableConnection>::Error>;

    /// Validate a connection before use
    async fn validate(&self, conn: &Self::Connection) -> bool {
        conn.is_valid().await
    }
}

/// Pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    /// Minimum number of connections to maintain
    pub min_size: usize,
    /// Maximum number of connections allowed
    pub max_size: usize,
    /// Maximum time to wait for a connection
    pub acquisition_timeout: Duration,
    /// Maximum idle time before closing a connection
    pub idle_timeout: Duration,
    /// Maximum lifetime of a connection
    pub max_lifetime: Duration,
    /// How often to run health checks
    pub health_check_interval: Duration,
    /// Whether to validate connections before use
    pub test_on_checkout: bool,
}

impl Default for PoolConfig {
    fn default() -> Self {
        Self {
            min_size: 1,
            max_size: 10,
            acquisition_timeout: Duration::from_secs(30),
            idle_timeout: Duration::from_secs(600), // 10 minutes
            max_lifetime: Duration::from_secs(3600), // 1 hour
            health_check_interval: Duration::from_secs(30),
            test_on_checkout: true,
        }
    }
}

impl PoolConfig {
    /// Validate the configuration
    ///
    /// # Errors
    ///
    /// Returns `PoolError::InvalidConfiguration` if `min_size` > `max_size` or `max_size` is 0.
    pub fn validate(&self) -> Result<(), PoolError> {
        if self.min_size > self.max_size {
            return Err(PoolError::InvalidConfiguration {
                message: "min_size cannot be greater than max_size".to_string(),
            });
        }

        if self.max_size == 0 {
            return Err(PoolError::InvalidConfiguration {
                message: "max_size must be greater than 0".to_string(),
            });
        }

        Ok(())
    }
}

/// Connection wrapper with metadata
struct PooledConnection<C: PoolableConnection> {
    connection: C,
    created_at: Instant,
    last_used: Instant,
    use_count: u64,
}

impl<C: PoolableConnection> PooledConnection<C> {
    fn new(connection: C) -> Self {
        let now = Instant::now();
        Self {
            connection,
            created_at: now,
            last_used: now,
            use_count: 0,
        }
    }

    fn is_expired(&self, config: &PoolConfig) -> bool {
        let now = Instant::now();

        // Check max lifetime
        if now.duration_since(self.created_at) > config.max_lifetime {
            return true;
        }

        // Check idle timeout
        if now.duration_since(self.last_used) > config.idle_timeout {
            return true;
        }

        false
    }

    fn mark_used(&mut self) {
        self.last_used = Instant::now();
        self.use_count += 1;
    }
}

/// Connection pool implementation
pub struct ConnectionPool<F: ConnectionFactory> {
    factory: Arc<F>,
    config: PoolConfig,
    connections: Arc<Mutex<Vec<PooledConnection<F::Connection>>>>,
    semaphore: Arc<Semaphore>,
    shutdown: Arc<Mutex<bool>>,
}

impl<F: ConnectionFactory> ConnectionPool<F> {
    /// Create a new connection pool
    ///
    /// # Errors
    ///
    /// Returns `PoolError::InvalidConfiguration` if the configuration is invalid.
    /// Returns `PoolError::CreationFailed` if initial connections cannot be created.
    pub async fn new(factory: F, config: PoolConfig) -> Result<Self, PoolError> {
        config.validate()?;

        let pool = Self {
            factory: Arc::new(factory),
            config: config.clone(),
            connections: Arc::new(Mutex::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(config.max_size)),
            shutdown: Arc::new(Mutex::new(false)),
        };

        // Create initial connections
        pool.ensure_minimum_connections().await?;

        // Start background health check task
        pool.start_health_check_task();

        Ok(pool)
    }

    /// Acquire a connection from the pool
    ///
    /// # Errors
    ///
    /// Returns `PoolError::ShuttingDown` if the pool is shutting down.
    /// Returns `PoolError::Exhausted` if no connections are available within the timeout.
    /// Returns `PoolError::CreationFailed` if a new connection cannot be created.
    pub async fn acquire(&self) -> Result<PoolGuard<F>, PoolError> {
        // Check if shutting down
        if *self.shutdown.lock().await {
            return Err(PoolError::ShuttingDown);
        }

        // Acquire semaphore permit with timeout
        let Ok(Ok(permit)) = tokio::time::timeout(
            self.config.acquisition_timeout,
            self.semaphore.clone().acquire_owned(),
        )
        .await
        else {
            return Err(PoolError::Exhausted);
        };

        // Try to get an existing connection
        loop {
            let pooled = {
                let mut connections = self.connections.lock().await;
                connections.pop()
            };

            match pooled {
                Some(mut p) => {
                    // Check if connection is still valid
                    if p.is_expired(&self.config) {
                        debug!("Connection expired, closing");
                        let _ = p.connection.close().await;
                        continue;
                    }

                    // Validate connection if configured
                    if self.config.test_on_checkout && !self.factory.validate(&p.connection).await {
                        debug!("Connection validation failed, closing");
                        let _ = p.connection.close().await;
                        continue;
                    }

                    p.mark_used();
                    let connection = p.connection;

                    return Ok(PoolGuard {
                        pool: self.clone(),
                        connection: Some(connection),
                        permit: Some(permit),
                    });
                }
                None => break, // No more connections in pool
            }
        }

        // Create a new connection
        match self.factory.create().await {
            Ok(connection) => {
                debug!("Created new connection");
                Ok(PoolGuard {
                    pool: self.clone(),
                    connection: Some(connection),
                    permit: Some(permit),
                })
            }
            Err(e) => Err(PoolError::CreationFailed {
                message: e.to_string(),
            }),
        }
    }

    /// Return a connection to the pool
    async fn return_connection(&self, mut connection: F::Connection) {
        // Check if shutting down
        if *self.shutdown.lock().await {
            let _ = connection.close().await;
            return;
        }

        // Reset the connection
        if let Err(e) = connection.reset().await {
            warn!("Failed to reset connection: {}", e);
            let _ = connection.close().await;
            return;
        }

        // Return to pool
        let mut connections = self.connections.lock().await;
        connections.push(PooledConnection::new(connection));
    }

    /// Ensure minimum connections are maintained
    ///
    /// # Errors
    ///
    /// Currently always returns Ok(()) but may return errors in the future.
    async fn ensure_minimum_connections(&self) -> Result<(), PoolError> {
        let connections = self.connections.lock().await;
        let current_size = connections.len();

        if current_size < self.config.min_size {
            let needed = self.config.min_size - current_size;
            drop(connections); // Release lock

            for _ in 0..needed {
                match self.factory.create().await {
                    Ok(connection) => {
                        let mut connections = self.connections.lock().await;
                        connections.push(PooledConnection::new(connection));
                    }
                    Err(e) => {
                        warn!("Failed to create minimum connection: {}", e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Start background health check task
    fn start_health_check_task(&self) {
        let pool = self.clone();
        let interval = self.config.health_check_interval;

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(interval);

            loop {
                interval.tick().await;

                // Check if shutting down
                if *pool.shutdown.lock().await {
                    break;
                }

                // Clean up expired connections
                let mut connections = pool.connections.lock().await;
                let mut to_remove = Vec::new();

                for (i, pooled) in connections.iter().enumerate() {
                    if pooled.is_expired(&pool.config) || !pooled.connection.is_valid().await {
                        to_remove.push(i);
                    }
                }

                // Remove invalid connections
                for i in to_remove.into_iter().rev() {
                    let pooled = connections.remove(i);
                    drop(connections); // Release lock
                    let _ = pooled.connection.close().await;
                    connections = pool.connections.lock().await;
                }

                drop(connections); // Release lock

                // Ensure minimum connections
                let _ = pool.ensure_minimum_connections().await;

                debug!("Health check completed");
            }

            info!("Health check task stopped");
        });
    }

    /// Get current pool statistics
    pub async fn stats(&self) -> PoolStats {
        let connections = self.connections.lock().await;
        let available = connections.len();
        let in_use = self.config.max_size - self.semaphore.available_permits();

        PoolStats {
            available,
            in_use,
            total: available + in_use,
            max_size: self.config.max_size,
        }
    }

    /// Shutdown the pool
    pub async fn shutdown(&self) {
        info!("Shutting down connection pool");

        // Mark as shutting down
        *self.shutdown.lock().await = true;

        // Close all connections
        let mut connections = self.connections.lock().await;
        for pooled in connections.drain(..) {
            let _ = pooled.connection.close().await;
        }
    }
}

impl<F: ConnectionFactory> Clone for ConnectionPool<F> {
    fn clone(&self) -> Self {
        Self {
            factory: self.factory.clone(),
            config: self.config.clone(),
            connections: self.connections.clone(),
            semaphore: self.semaphore.clone(),
            shutdown: self.shutdown.clone(),
        }
    }
}

/// Guard for pooled connections that returns them when dropped
pub struct PoolGuard<F: ConnectionFactory> {
    pool: ConnectionPool<F>,
    connection: Option<F::Connection>,
    #[allow(dead_code)] // Kept alive to maintain semaphore permit
    permit: Option<tokio::sync::OwnedSemaphorePermit>,
}

impl<F: ConnectionFactory> PoolGuard<F> {
    /// Get a reference to the connection
    ///
    /// # Panics
    ///
    /// Panics if the connection has already been taken.
    #[must_use]
    pub fn get(&self) -> &F::Connection {
        self.connection.as_ref().expect("Connection already taken")
    }

    /// Get a mutable reference to the connection
    ///
    /// # Panics
    ///
    /// Panics if the connection has already been taken.
    #[must_use]
    pub fn get_mut(&mut self) -> &mut F::Connection {
        self.connection.as_mut().expect("Connection already taken")
    }

    /// Take the connection, preventing it from being returned to the pool
    ///
    /// # Panics
    ///
    /// Panics if the connection has already been taken.
    pub fn take(mut self) -> F::Connection {
        self.connection.take().expect("Connection already taken")
    }
}

impl<F: ConnectionFactory> Drop for PoolGuard<F> {
    fn drop(&mut self) {
        if let Some(connection) = self.connection.take() {
            let pool = self.pool.clone();
            tokio::spawn(async move {
                pool.return_connection(connection).await;
            });
        }
    }
}

impl<F: ConnectionFactory> std::ops::Deref for PoolGuard<F> {
    type Target = F::Connection;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

impl<F: ConnectionFactory> std::ops::DerefMut for PoolGuard<F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}

/// Pool statistics
#[derive(Debug, Clone)]
pub struct PoolStats {
    /// Number of available connections in the pool
    pub available: usize,
    /// Number of connections currently in use
    pub in_use: usize,
    /// Total number of connections (available + in use)
    pub total: usize,
    /// Maximum allowed pool size
    pub max_size: usize,
}

/// Builder pattern for connection pools
pub struct PoolBuilder<F: ConnectionFactory> {
    factory: F,
    config: PoolConfig,
}

impl<F: ConnectionFactory> PoolBuilder<F> {
    /// Create a new pool builder
    #[must_use]
    pub fn new(factory: F) -> Self {
        Self {
            factory,
            config: PoolConfig::default(),
        }
    }

    /// Set minimum pool size
    #[must_use]
    pub fn min_size(mut self, size: usize) -> Self {
        self.config.min_size = size;
        self
    }

    /// Set maximum pool size
    #[must_use]
    pub fn max_size(mut self, size: usize) -> Self {
        self.config.max_size = size;
        self
    }

    /// Set acquisition timeout
    #[must_use]
    pub fn acquisition_timeout(mut self, timeout: Duration) -> Self {
        self.config.acquisition_timeout = timeout;
        self
    }

    /// Set idle timeout
    #[must_use]
    pub fn idle_timeout(mut self, timeout: Duration) -> Self {
        self.config.idle_timeout = timeout;
        self
    }

    /// Set maximum lifetime
    #[must_use]
    pub fn max_lifetime(mut self, lifetime: Duration) -> Self {
        self.config.max_lifetime = lifetime;
        self
    }

    /// Enable or disable test on checkout
    #[must_use]
    pub fn test_on_checkout(mut self, test: bool) -> Self {
        self.config.test_on_checkout = test;
        self
    }

    /// Build the pool
    ///
    /// # Errors
    ///
    /// Returns `PoolError::InvalidConfiguration` if the configuration is invalid.
    /// Returns `PoolError::CreationFailed` if initial connections cannot be created.
    pub async fn build(self) -> Result<ConnectionPool<F>, PoolError> {
        ConnectionPool::new(self.factory, self.config).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    // Mock connection for testing
    struct MockConnection {
        id: u32,
        valid: Arc<AtomicBool>,
        closed: Arc<AtomicBool>,
    }

    #[async_trait::async_trait]
    impl PoolableConnection for MockConnection {
        type Error = std::io::Error;

        async fn is_valid(&self) -> bool {
            self.valid.load(Ordering::SeqCst)
        }

        async fn reset(&mut self) -> Result<(), Self::Error> {
            Ok(())
        }

        async fn close(self) -> Result<(), Self::Error> {
            self.closed.store(true, Ordering::SeqCst);
            Ok(())
        }
    }

    struct MockFactory {
        counter: Arc<AtomicU32>,
        valid: Arc<AtomicBool>,
    }

    #[async_trait::async_trait]
    impl ConnectionFactory for MockFactory {
        type Connection = MockConnection;

        async fn create(&self) -> Result<Self::Connection, std::io::Error> {
            let id = self.counter.fetch_add(1, Ordering::SeqCst);
            Ok(MockConnection {
                id,
                valid: self.valid.clone(),
                closed: Arc::new(AtomicBool::new(false)),
            })
        }
    }

    #[tokio::test]
    async fn test_pool_creation() {
        let factory = MockFactory {
            counter: Arc::new(AtomicU32::new(0)),
            valid: Arc::new(AtomicBool::new(true)),
        };

        let pool = PoolBuilder::new(factory)
            .min_size(2)
            .max_size(5)
            .build()
            .await
            .unwrap();

        let stats = pool.stats().await;
        assert_eq!(stats.available, 2);
        assert_eq!(stats.in_use, 0);
    }

    #[tokio::test]
    async fn test_connection_acquisition() {
        let factory = MockFactory {
            counter: Arc::new(AtomicU32::new(0)),
            valid: Arc::new(AtomicBool::new(true)),
        };

        let pool = PoolBuilder::new(factory).max_size(3).build().await.unwrap();

        // Acquire connections
        let conn1 = pool.acquire().await.unwrap();
        let conn2 = pool.acquire().await.unwrap();

        assert_eq!(conn1.id, 0);
        assert_eq!(conn2.id, 1);

        let stats = pool.stats().await;
        assert_eq!(stats.in_use, 2);

        // Return connections
        drop(conn1);
        drop(conn2);

        // Wait for async return
        tokio::time::sleep(Duration::from_millis(100)).await;

        let stats = pool.stats().await;
        assert_eq!(stats.available, 2);
    }

    #[tokio::test]
    async fn test_pool_exhaustion() {
        let factory = MockFactory {
            counter: Arc::new(AtomicU32::new(0)),
            valid: Arc::new(AtomicBool::new(true)),
        };

        let pool = PoolBuilder::new(factory)
            .max_size(2)
            .acquisition_timeout(Duration::from_millis(100))
            .build()
            .await
            .unwrap();

        // Acquire all connections
        let _conn1 = pool.acquire().await.unwrap();
        let _conn2 = pool.acquire().await.unwrap();

        // Try to acquire another
        let result = pool.acquire().await;
        assert!(matches!(result, Err(PoolError::Exhausted)));
    }

    #[tokio::test]
    async fn test_invalid_connection_handling() {
        let valid = Arc::new(AtomicBool::new(true));
        let factory = MockFactory {
            counter: Arc::new(AtomicU32::new(0)),
            valid: valid.clone(),
        };

        let pool = PoolBuilder::new(factory)
            .test_on_checkout(true)
            .build()
            .await
            .unwrap();

        // Acquire and return a connection
        let conn = pool.acquire().await.unwrap();
        drop(conn);

        // Wait for return
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Invalidate connections
        valid.store(false, Ordering::SeqCst);

        // Next acquisition should create a new connection
        let conn = pool.acquire().await.unwrap();
        assert_eq!(conn.id, 1); // Should be a new connection
    }
}
