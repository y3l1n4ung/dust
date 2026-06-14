use dust_ir::{ConfigApplicationIr, SymbolId};
use dust_parser_dart::ParsedAnnotation;

#[cfg(test)]
use dust_ir::SpanIr;
#[cfg(test)]
use dust_text::{FileId, TextRange};

use super::{constants::VIEW_MODEL, model::ViewModelAnnotation};

pub(crate) fn view_model_config(configs: &[ConfigApplicationIr]) -> Option<&ConfigApplicationIr> {
    configs
        .iter()
        .find(|config| config_name(&config.symbol) == VIEW_MODEL)
}

#[cfg(test)]
pub(crate) fn parse_view_model_annotation(args: Option<&str>) -> Option<ViewModelAnnotation> {
    parse_view_model_config(&test_config(args))
}

pub(crate) fn parse_view_model_config(config: &ConfigApplicationIr) -> Option<ViewModelAnnotation> {
    let state_type = config
        .named_type("state")
        .or_else(|| config.positional_type(0))?;
    let args_type = config.named_type("args");
    let initial_source = config.named_expression_source("initial");
    Some(ViewModelAnnotation {
        state_type,
        args_type,
        initial_source,
    })
}

pub(crate) fn parse_view_model_surface(
    annotation: &ParsedAnnotation,
) -> Option<ViewModelAnnotation> {
    let state_type = annotation
        .named_type("state")
        .or_else(|| annotation.positional_type(0))?;
    let args_type = annotation.named_type("args");
    let initial_source = annotation.named_expression_source("initial");
    Some(ViewModelAnnotation {
        state_type,
        args_type,
        initial_source,
    })
}

fn config_name(symbol: &SymbolId) -> &str {
    symbol.0.rsplit("::").next().unwrap_or(symbol.0.as_str())
}

#[cfg(test)]
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
