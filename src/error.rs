use crate::hook::ExtensionPointID;

/// Result type for plugin operations
pub type HookResult<T> = Result<T, HookError>;

/// Result type for plugin operations
pub type PluginResult<T> = Result<T, PluginError>;

/// Error type for plugin operations
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("{0} was already loaded")]
    AlreadyLoaded(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Plugin execution error: {0}")]
    ExecutionError(#[from] anyhow::Error),

    #[error("Hook error: {0}")]
    HookError(#[from] HookError),
}

/// Error type for plugin operations
#[derive(Debug, thiserror::Error)]
pub enum HookError {
    #[error("{0} was already loaded")]
    AlreadyLoaded(String),

    #[error("Plugin not found: {0}")]
    NotFound(String),

    #[error("Hook not found: {0}")]
    HookNotFound(ExtensionPointID),

    #[error("Plugin execution error: {0}")]
    ExecutionError(anyhow::Error),

    #[error("Tried to register a hook for a ExtensionPointID that is not supported by the plugin manager")]
    NotSupported,
}
