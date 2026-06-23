/// Dust annotation name that enables view model support generation.
pub(crate) const VIEW_MODEL: &str = "ViewModel";
/// Workspace analysis key containing serialized view model facts.
pub(crate) const VIEW_MODELS_ANALYSIS_KEY: &str = "dust_state.view_models.v1";
/// Workspace analysis key containing serialized state field facts.
pub(crate) const STATES_ANALYSIS_KEY: &str = "dust_state.states.v1";

/// Configuration symbols claimed by the state plugin.
pub(crate) const CLAIMED_CONFIG_SYMBOLS: &[&str] = &["dust_flutter::ViewModel"];
/// Annotation names supported directly by the state plugin.
pub(crate) const SUPPORTED_ANNOTATIONS: &[&str] = &[VIEW_MODEL];
