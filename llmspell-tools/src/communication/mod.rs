//! ABOUTME: Communication tools for email sending and database connectivity
//! ABOUTME: Provides `EmailSenderTool` and `DatabaseConnectorTool` with multiple provider support

pub mod database_connector;
pub mod email_sender;

pub use database_connector::DatabaseConnectorTool;
pub use email_sender::EmailSenderTool;
