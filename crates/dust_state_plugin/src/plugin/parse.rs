use dust_ir::{ConfigApplicationIr, SymbolId};
use dust_parser_dart::ParsedAnnotation;

#[cfg(test)]
use dust_dart_emit::{normalized_args, parse_named_arguments, split_top_level_items};

use super::{constants::VIEW_MODEL, model::ViewModelAnnotation};

pub(crate) fn view_model_config(configs: &[ConfigApplicationIr]) -> Option<&ConfigApplicationIr> {
    configs
        .iter()
        .find(|config| config_name(&config.symbol) == VIEW_MODEL)
}

#[cfg(test)]
pub(crate) fn parse_view_model_annotation(args: Option<&str>) -> Option<ViewModelAnnotation> {
    let args = args?;
    let state_type = named_type_literal(args, "state").or_else(|| first_type_literal(args))?;
    let args_type = named_type_literal(args, "args");
    let initial_source = named_value_source(args, "initial").map(str::to_owned);
    Some(ViewModelAnnotation {
        state_type,
        args_type,
        initial_source,
    })
}

pub(crate) fn parse_view_model_config(config: &ConfigApplicationIr) -> Option<ViewModelAnnotation> {
    let state_type = config
        .named_argument_source("state")
        .and_then(parse_type_name)
        .or_else(|| {
            config
                .positional_argument_source(0)
                .and_then(parse_type_name)
        })?;
    let args_type = config
        .named_argument_source("args")
        .and_then(parse_type_name);
    let initial_source = config.named_argument_source("initial").map(str::to_owned);
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
        .named_argument_source("state")
        .and_then(parse_type_name)
        .or_else(|| {
            annotation
                .positional_argument_source(0)
                .and_then(parse_type_name)
        })?;
    let args_type = annotation
        .named_argument_source("args")
        .and_then(parse_type_name);
    let initial_source = annotation
        .named_argument_source("initial")
        .map(str::to_owned);
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
fn first_type_literal(args: &str) -> Option<String> {
    let first = split_top_level_items(normalized_args(args)?)
        .first()
        .copied()?;
    if first.contains(':') {
        return None;
    }
    parse_type_name(first)
}

#[cfg(test)]
fn named_type_literal(args: &str, name: &str) -> Option<String> {
    let value = named_value_source(args, name)?.trim();
    parse_type_name(value)
}

fn parse_type_name(value: &str) -> Option<String> {
    let ident = value
        .trim()
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '.')
        .collect::<String>();
    (!ident.is_empty()).then_some(ident)
}

#[cfg(test)]
fn named_value_source<'a>(args: &'a str, name: &str) -> Option<&'a str> {
    for (key, value) in parse_named_arguments(Some(args)) {
        if key.trim() == name {
            return Some(value.trim());
        }
    }
    None
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
