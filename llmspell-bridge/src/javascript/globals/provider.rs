//! ABOUTME: JavaScript-specific Provider global stub implementation
//! ABOUTME: Placeholder for future JavaScript provider support (Phase 2)

use llmspell_core::error::LLMSpellError;

/// Inject Provider global stub into JavaScript environment
///
/// This is a placeholder implementation that will be completed in Phase 2
/// when JavaScript provider support is fully implemented.
///
/// # Errors
///
/// Currently always returns an error indicating JavaScript provider support
/// is not yet implemented.
pub const fn inject_provider_global_stub() -> Result<(), LLMSpellError> {
    // TODO: Implement JavaScript provider support in Phase 2
    // This will mirror the Lua implementation with:
    // - Provider.list() - List all available providers
    // - Provider.get(name) - Get specific provider information
    // - Provider.getCapabilities(name) - Get provider capabilities
    // - Provider.isAvailable(name) - Check if provider is configured

    // For now, we just return Ok to allow the system to continue
    // Scripts trying to use Provider will get undefined behavior
    Ok(())
}
