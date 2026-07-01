use dust_ir::{ConfigApplicationIr, SymbolId};
use dust_parser_dart::ParsedAnnotation;

#[cfg(test)]
use dust_ir::SpanIr;
#[cfg(test)]
use dust_text::{FileId, TextRange};

use super::{
    constants::VIEW_MODEL,
    model::{ViewModelAnnotation, ViewModelMode},
};

/// Finds the first `@ViewModel` config application in a class config list.
pub(crate) fn view_model_config(configs: &[ConfigApplicationIr]) -> Option<&ConfigApplicationIr> {
    configs
        .iter()
        .find(|config| config_name(&config.symbol) == VIEW_MODEL)
}

#[cfg(test)]
/// Parses a synthetic `@ViewModel` annotation argument list for unit tests.
pub(crate) fn parse_view_model_annotation(args: Option<&str>) -> Option<ViewModelAnnotation> {
    parse_view_model_config(&test_config(args))
}

/// Extracts state, args, and initial values from resolved view model config IR.
pub(crate) fn parse_view_model_config(config: &ConfigApplicationIr) -> Option<ViewModelAnnotation> {
    let state_type = config
        .named_type("state")
        .or_else(|| config.positional_type(0))?;
    let args_type = config.named_type("args");
    let initial_source = config.named_expression_source("initial");
    let mode_source = config.named_expression_source("mode");
    let mode = parse_mode(mode_source.as_deref());
    Some(ViewModelAnnotation {
        state_type,
        args_type,
        initial_source,
        mode_source,
        mode,
    })
}

/// Extracts state, args, and initial values from parser-level annotation data.
pub(crate) fn parse_view_model_surface(
    annotation: &ParsedAnnotation,
) -> Option<ViewModelAnnotation> {
    let state_type = annotation
        .named_type("state")
        .or_else(|| annotation.positional_type(0))?;
    let args_type = annotation.named_type("args");
    let initial_source = annotation.named_expression_source("initial");
    let mode_source = annotation.named_expression_source("mode");
    let mode = parse_mode(mode_source.as_deref());
    Some(ViewModelAnnotation {
        state_type,
        args_type,
        initial_source,
        mode_source,
        mode,
    })
}

pub(crate) fn parse_mode(source: Option<&str>) -> ViewModelMode {
    match source.map(str::trim) {
        Some(source) if source.ends_with(".async") || source == "async" => ViewModelMode::Async,
        _ => ViewModelMode::Sync,
    }
}

pub(crate) fn is_valid_mode_source(source: &str) -> bool {
    let source = source.trim();
    source == "sync" || source == "async" || source.ends_with(".sync") || source.ends_with(".async")
}

/// Returns the final unqualified segment of a config symbol identifier.
fn config_name(symbol: &SymbolId) -> &str {
    symbol.0.rsplit("::").next().unwrap_or(symbol.0.as_str())
}

#[cfg(test)]
/// Builds a resolved config IR value around a raw annotation argument list.
fn test_config(args: Option<&str>) -> ConfigApplicationIr {
    ConfigApplicationIr::new(
        SymbolId::new(VIEW_MODEL),
        args.map(str::to_owned),
        SpanIr::new(FileId::default(), TextRange::default()),
    )
}

#[cfg(test)]
mod tests {
    use super::parse_view_model_annotation;

    #[test]
    fn parses_named_state_and_args() {
        let annotation =
            parse_view_model_annotation(Some("(state: TaskBoardState, args: TaskBoardArgs)"))
                .unwrap();
        assert_eq!(annotation.state_type, "TaskBoardState");
        assert_eq!(annotation.args_type.as_deref(), Some("TaskBoardArgs"));
        assert_eq!(annotation.initial_source, None);
        assert_eq!(annotation.mode, super::ViewModelMode::Sync);
    }

    #[test]
    fn parses_async_mode() {
        let annotation = parse_view_model_annotation(Some(
            "(state: Profile, args: ProfileArgs, mode: ViewModelMode.async)",
        ))
        .unwrap();
        assert_eq!(annotation.state_type, "Profile");
        assert_eq!(annotation.args_type.as_deref(), Some("ProfileArgs"));
        assert_eq!(annotation.mode, super::ViewModelMode::Async);
    }
}

#[cfg(test)]
mod initial_tests {
    use super::parse_view_model_annotation;

    #[test]
    fn parses_initial_expression_source() {
        let annotation = parse_view_model_annotation(Some(
            "(state: ShellTab, args: ShellViewModelArgs, initial: ShellTab.dashboard)",
        ))
        .unwrap();
        assert_eq!(
            annotation.initial_source.as_deref(),
            Some("ShellTab.dashboard")
        );
    }
}
