/// Validates class-level HTTP client shape.
mod class;
/// Validates endpoint methods.
mod endpoint;
/// Runs cross-parameter validation after endpoint scanning.
mod finalize;
/// Validates static header declarations.
mod header;
/// Validates model JSON support required by generated HTTP code.
mod json_capability;
/// Tracks and validates HTTP parameter annotations.
mod param;

pub(super) use class::validate_client_class;
pub(super) use endpoint::validate_endpoint;
pub(super) use json_capability::{JsonCapabilityContext, collect_workspace_analysis};
