use serde::{Deserialize, Serialize};

/// Generated ViewModel base mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ViewModelMode {
    /// Synchronous state managed directly by the ViewModel.
    Sync,
    /// Async loaded data wrapped in generated lifecycle state.
    Async,
}

/// Workspace fact describing a view model class discovered during parsing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct ViewModelFact {
    /// Name of the user-authored view model class.
    pub(crate) class_name: String,
    /// Dart type source for the state managed by the view model.
    pub(crate) state_type: String,
    /// Optional Dart type source for route or construction arguments.
    pub(crate) args_type: Option<String>,
    /// Optional Dart expression source used as the initial state value.
    pub(crate) initial_source: Option<String>,
    /// Generated ViewModel base mode.
    pub(crate) mode: ViewModelMode,
    /// Name of the generated abstract base class the view model must extend.
    pub(crate) generated_base_class: String,
    /// Import URI that makes the view model visible from other libraries.
    pub(crate) import_uri: String,
}

/// Parsed `@ViewModel` annotation arguments for one class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ViewModelAnnotation {
    /// Dart type source named by the `state` argument.
    pub(crate) state_type: String,
    /// Optional Dart type source named by the `args` argument.
    pub(crate) args_type: Option<String>,
    /// Optional Dart expression named by the `initial` argument.
    pub(crate) initial_source: Option<String>,
    /// Raw mode expression source, if supplied.
    pub(crate) mode_source: Option<String>,
    /// Generated ViewModel base mode.
    pub(crate) mode: ViewModelMode,
}

/// Workspace fact describing a state class and the fields available to selectors.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StateFact {
    /// Name of the state class.
    pub(crate) class_name: String,
    /// Fields read from the state class.
    pub(crate) fields: Vec<StateFieldFact>,
}

/// Serializable description of one field on a state class.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(crate) struct StateFieldFact {
    /// Dart field name.
    pub(crate) name: String,
    /// Sanitized Dart type source safe to emit into generated selectors.
    pub(crate) type_source: String,
}
