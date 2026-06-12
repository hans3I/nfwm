//! Tiling errors: structured errors from tiling operations.
//!
//! These map directly to the legacy `TilingError` enum.

/// Error cases for tiling operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum TilingError {
    #[error("Operation failed")]
    Failed,
    #[error("No focused window")]
    MissingTarget,
    #[error("Invalid target for operation")]
    InvalidTarget,
    #[error("No adjacent window")]
    MissingAdjacentWindow,
    #[error("Operation would cause recursive nesting")]
    CausesRecursiveNesting,
    #[error("Cannot modify the top-level panel")]
    ModifiesTopLevelPanel,
    #[error("No valid placement exists")]
    NoValidPlacementExists,
    #[error("Target window cannot fit in the available space")]
    TargetCannotFit,
    #[error("Cannot pull beyond the top-level panel")]
    PullsBeyondTopLevelPanel,
    #[error("Nesting inside a stack panel is not allowed")]
    NestingInStackPanel,
    #[error("Tiling is not active")]
    NotActive,
    #[error("Window is not managed")]
    NotManaged,
    #[error("Window is already floating")]
    AlreadyFloating,
}

/// Result type for tiling operations.
pub type TilingResult<T> = Result<T, TilingError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tiling_error_display() {
        assert_eq!(
            format!("{}", TilingError::MissingTarget),
            "No focused window"
        );
        assert_eq!(
            format!("{}", TilingError::NoValidPlacementExists),
            "No valid placement exists"
        );
    }
}
