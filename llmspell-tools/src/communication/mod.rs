//! ABOUTME: Communication tools for email sending and database connectivity
//! ABOUTME: Provides `EmailSenderTool` and `DatabaseConnectorTool` with multiple provider support

#[cfg(feature = "database")]
pub mod database_connector;
#[cfg(feature = "email")]
pub mod email_sender;

#[cfg(feature = "database")]
pub use database_connector::DatabaseConnectorTool;
#[cfg(feature = "email")]
pub use email_sender::EmailSenderTool;
