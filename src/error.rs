//! # Error Types
//!
//! This module provides error types for the [`steckrs`](crate) library.
//!
//! The error types in this module are used throughout the library to represent
//! various failure conditions that can occur during plugin management, hook
//! registration, and other operations.
//!
//! Error types are defined with help of the [`thiserror`] crate.
//!
//! ## Error Types
//!
//! - [`PluginError`]: Errors related to [Plugin](crate::Plugin) management
//! - [`HookError`]: Errors related to [hook](crate::hook::Hook) registration and management
//!
//! ## Result Types
//!
//! This module also provides type aliases for commonly used Result types:
//!
//! - [`PluginResult<T>`]: Results from plugin operations
//! - [`HookResult<T>`]: Results from hook operations

use crate::PluginID;

/// Result type for plugin operations
pub type HookResult<T> = Result<T, HookError>;

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;

/// Error type for plugin-related operations.
///
/// These errors can occur during plugin loading, unloading, enabling,
/// disabling, or other plugin management operations.
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    /// A [Plugin](crate::Plugin) is already loaded
    #[error("{0} was already loaded")]
    AlreadyLoaded(PluginID),

    /// A [Plugin](crate::Plugin) was requested that is not registered
    #[error("Plugin not found: {0}")]
    NotFound(PluginID),

    /// Something went wrong when working with hooks
    #[error("Hook error: {0}")]
    HookError(#[from] HookError),
}

/// Error type for hook-related operations.
///
/// These errors can occur during hook registration, deregistration,
/// or other hook management operations.
#[derive(Debug, thiserror::Error)]
pub enum HookError {
    /// Indicates a hook with the same ID is already registered.
    #[error("Tried to register to a hook that already exists")]
    AlreadyRegistered,
}
