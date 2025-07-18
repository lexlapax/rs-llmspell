//! ABOUTME: Agent lifecycle management system
//! ABOUTME: Comprehensive lifecycle management including state machines, events, resources, and shutdown coordination

pub mod events;
pub mod hooks;
pub mod middleware;
pub mod resources;
pub mod shutdown;
pub mod state_machine;

// Re-export all lifecycle components
pub use events::*;
pub use hooks::*;
pub use middleware::*;
pub use resources::*;
pub use shutdown::*;
pub use state_machine::*;
