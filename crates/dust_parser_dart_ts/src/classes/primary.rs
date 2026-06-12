use dust_parser_dart::{
    ParameterKind, ParsedAnnotation, ParsedClassKind, ParsedClassSurface,
    ParsedConstructorParamSurface, ParsedConstructorSurface, ParsedFieldSurface,
};
use dust_text::SourceText;

use super::primary_text::{
    find_keyword, find_matching, has_word, range, read_identifier, skip_type_arguments, skip_ws,
    split_default, split_top_level,
};

pub(crate) fn extract_primary_constructor_classes(source: &SourceText) -> Vec<ParsedClassSurface> {
    let text = source.as_str();
    let mut classes = Vec::new();
    let mut offset = 0;

    while let Some(class_index) = find_keyword(text, "class", offset) {
        let Some(parsed) = parse_primary_class(text, class_index) else {
            offset = class_index + "class".len();
            continue;
        };
        offset = parsed
            .span
            .end()
            .to_usize()
            .max(class_index + "class".len());
        classes.push(parsed);
    }

    classes
}

fn parse_primary_class(text: &str, class_index: usize) -> Option<ParsedClassSurface> {
    let mut index = skip_ws(text, class_index + "class".len());
    if text.get(index..)?.starts_with("const ") {
        index = skip_ws(text, index + "const".len());
    }

    let name_start = index;
    let name = read_identifier(text, &mut index)?;
    index = skip_type_arguments(text, skip_ws(text, index));
    index = skip_ws(text, index);
    if !text.get(index..)?.starts_with('(') {
        return None;
    }

    let params_start = index;
    let params_end = find_matching(text, params_start, '(', ')')?;
    let after_params = skip_ws(text, params_end + 1);
    let end = if text.get(after_params..)?.starts_with(';') {
        after_params + 1
    } else if text.get(after_params..)?.starts_with('{') {
        find_matching(text, after_params, '{', '}')? + 1
    } else {
        return None;
    };
    let end = skip_ws(text, end);

    let header_start = annotation_start(text, class_index);
    let header = text.get(header_start..params_start)?.trim();
    let params_text = text.get(params_start + 1..params_end)?;
    let primary_params = parse_params(params_text, params_start + 1);
    let fields = primary_params
        .iter()
        .filter(|param| param.declares_field)
        .filter_map(|param| param_field(&param.surface))
        .collect::<Vec<ParsedFieldSurface>>();
    let params = primary_params
        .into_iter()
        .map(|param| param.surface)
        .collect::<Vec<_>>();

    Some(ParsedClassSurface {
        kind: if header.contains("mixin class") {
            ParsedClassKind::MixinClass
        } else {
            ParsedClassKind::Class
        },
        name,
        is_abstract: has_word(header, "abstract"),
        is_interface: header.contains("interface class"),
        superclass_name: None,
        annotations: parse_annotations(text, header_start, class_index),
        fields,
        constructors: vec![ParsedConstructorSurface {
            name: None,
            is_factory: false,
            redirected_target_source: None,
            redirected_target_name: None,
            params,
            span: range(class_index, end),
        }],
        methods: Vec::new(),
        span: range(header_start.min(name_start), end),
    })
}

struct PrimaryParam {
    surface: ParsedConstructorParamSurface,
    declares_field: bool,
}

fn parse_params(text: &str, base: usize) -> Vec<PrimaryParam> {
    split_top_level(text, ',')
        .into_iter()
        .flat_map(|(start, end)| parse_param_part(text, start, end, base))
        .collect()
}

fn parse_param_part(text: &str, start: usize, end: usize, base: usize) -> Vec<PrimaryParam> {
    let Some(raw) = text.get(start..end) else {
        return Vec::new();
    };
    let trimmed = raw.trim();
    if trimmed.starts_with('{') && trimmed.ends_with('}') {
        let leading_ws = raw.len() - raw.trim_start().len();
        let inner_start = start + leading_ws + 1;
        let inner_end = end.saturating_sub(raw.len() - raw.trim_end().len() + 1);
        return split_top_level(text.get(inner_start..inner_end).unwrap_or_default(), ',')
            .into_iter()
            .filter_map(|(part_start, part_end)| {
                parse_param(
                    text.get(inner_start + part_start..inner_start + part_end)?,
                    base + inner_start + part_start,
                    ParameterKind::Named,
                )
            })
            .collect();
    }

    parse_param(raw, base + start, ParameterKind::Positional)
        .into_iter()
        .collect()
}

fn parse_param(
    raw: &str,
    absolute_start: usize,
    fallback_kind: ParameterKind,
) -> Option<PrimaryParam> {
    let trimmed_start = raw.len() - raw.trim_start().len();
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }

    parse_param_inner(trimmed, absolute_start + trimmed_start, fallback_kind)
}

fn parse_param_inner(
    text: &str,
    absolute_start: usize,
    kind: ParameterKind,
) -> Option<PrimaryParam> {
    let default_value_source = split_default(text).map(|(_, value)| value.trim().to_owned());
    let before_default = split_default(text).map_or(text, |(head, _)| head).trim();
    let words = before_default.split_whitespace().collect::<Vec<_>>();
    let name = words.last()?.trim_start_matches("this.").to_owned();
    let type_source = param_type(&words);

    let declares_field = words.iter().any(|word| matches!(*word, "var" | "final"));

    Some(PrimaryParam {
        surface: ParsedConstructorParamSurface {
            name,
            type_source,
            kind,
            has_default: default_value_source.is_some(),
            default_value_source,
            span: range(absolute_start, absolute_start + text.len()),
        },
        declares_field,
    })
}

fn param_type(words: &[&str]) -> Option<String> {
    if words.len() < 2 {
        return None;
    }
    let type_words = words[..words.len() - 1]
        .iter()
        .copied()
        .filter(|word| !matches!(*word, "required" | "var" | "final" | "covariant"))
        .collect::<Vec<_>>();
    if type_words.is_empty() {
        None
    } else {
        Some(type_words.join(" "))
    }
}

fn param_field(param: &ParsedConstructorParamSurface) -> Option<ParsedFieldSurface> {
    let ty = param.type_source.clone()?;
    Some(ParsedFieldSurface {
        name: param.name.clone(),
        annotations: Vec::new(),
        type_source: Some(ty),
        has_default: param.has_default,
        span: param.span,
    })
}

fn parse_annotations(text: &str, start: usize, end: usize) -> Vec<ParsedAnnotation> {
    let Some(raw) = text.get(start..end) else {
        return Vec::new();
    };
    raw.lines()
        .scan(start, |line_start, line| {
            let current = *line_start;
            *line_start += line.len() + 1;
            Some((current, line.trim()))
        })
        .filter(|(_, line)| line.starts_with('@'))
        .filter_map(|(line_start, line)| parse_annotation_line(line, line_start))
        .collect()
}

fn parse_annotation_line(line: &str, line_start: usize) -> Option<ParsedAnnotation> {
    let source = line.strip_prefix('@')?.trim();
    let name_end = source
        .char_indices()
        .find(|(_, ch)| !(ch.is_ascii_alphanumeric() || *ch == '_' || *ch == '.'))
        .map_or(source.len(), |(index, _)| index);
    let name = source.get(..name_end)?.rsplit('.').next()?.to_owned();
    let arguments_source = source
        .get(name_end..)
        .map(str::trim)
        .filter(|arguments| arguments.starts_with('(') && arguments.ends_with(')'))
        .map(str::to_owned);
    Some(ParsedAnnotation {
        name,
        arguments_source,
        span: range(line_start, line_start + line.len()),
    })
}

fn annotation_start(text: &str, class_index: usize) -> usize {
    let mut start = text[..class_index].rfind('\n').map_or(0, |index| index + 1);
    loop {
        let previous_end = start.saturating_sub(1);
        let Some(previous_start) = text[..previous_end].rfind('\n').map(|index| index + 1) else {
            break;
        };
        let Some(line) = text.get(previous_start..previous_end) else {
            break;
        };
        if line.trim_start().starts_with('@') || line.trim().is_empty() {
            start = previous_start;
        } else {
            break;
        }
    }
    start
}
