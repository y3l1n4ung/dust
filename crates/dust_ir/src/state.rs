/// Resolver-normalized `ViewModel` configuration.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateConfigIr {
    /// Dart type source for the state managed by the view model.
    pub state_type: String,
    /// Optional Dart type source for construction arguments.
    pub args_type: Option<String>,
    /// Optional Dart expression source used as the initial state value.
    pub initial_source: Option<String>,
    /// Raw mode expression source, when one was supplied.
    pub mode_source: Option<String>,
    /// Normalized view model mode.
    pub mode: StateModeIr,
}

/// Normalized mode for a `ViewModel` configuration.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateModeIr {
    /// Synchronous state managed directly by the view model.
    Sync,
    /// Asynchronously loaded state managed by generated lifecycle support.
    Async,
}
