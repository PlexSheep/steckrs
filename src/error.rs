use crate::hook::ExtensionPointID;
use crate::PluginID;

/// Result type for plugin operations
pub type HookResult<T> = Result<T, HookError>;

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;

/// Error type for plugin operations
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("{0} was already loaded")]
    AlreadyLoaded(PluginID),

    #[error("Plugin not found: {0}")]
    NotFound(PluginID),

    #[error("Hook error: {0}")]
    HookError(#[from] HookError),
}

/// Error type for plugin operations
#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error("Hook not found: {0}")]
    HookNotFound(ExtensionPointID),
    #[error("Tried to register to a hook that already exists")]
    AlreadyRegistered,
}
