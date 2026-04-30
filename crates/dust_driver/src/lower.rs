use std::collections::HashMap;

use dust_diagnostics::Diagnostic;
use dust_ir::{
    BuiltinType, ClassIr, ConfigApplicationIr, ConstructorIr, ConstructorParamIr, EnumIr,
    EnumVariantIr, FieldIr, LibraryIr, LoweringOutcome, ParamKind, SerdeClassConfigIr,
    SerdeFieldConfigIr, SerdeRenameRuleIr, SpanIr, TypeIr,
};
use dust_parser_dart::ParameterKind;
use dust_resolver::{ResolvedClass, ResolvedLibrary};

/// Lowers one resolved library into semantic IR.
pub(crate) fn lower_library(library: &ResolvedLibrary) -> LoweringOutcome<LibraryIr> {
    let mut diagnostics = Vec::new();
    let mut classes = library
        .classes
        .iter()
        .filter(|class| !class.traits.is_empty() || !class.configs.is_empty())
        .map(|class| {
            let outcome = lower_class(class);
            diagnostics.extend(outcome.diagnostics);
            outcome.value
        })
        .collect::<Vec<_>>();
    let enums = library
        .enums
        .iter()
        .map(|e| {
            let outcome = lower_enum(e);
            diagnostics.extend(outcome.diagnostics);
            outcome.value
        })
        .collect();

    let index_by_name = classes
        .iter()
        .enumerate()
        .map(|(index, class)| (class.name.clone(), index))
        .collect::<HashMap<_, _>>();
    let mut merged_cache = HashMap::new();
    let mut active_stack = Vec::new();
    for index in 0..classes.len() {
        let merged_fields = merged_fields_for_class(
            index,
            &classes,
            &index_by_name,
            &mut merged_cache,
            &mut active_stack,
            &mut diagnostics,
        );
        classes[index].fields = merged_fields;
        resolve_constructor_param_types(&mut classes[index], &mut diagnostics);
    }

    LoweringOutcome {
        value: LibraryIr {
            source_path: library.source_path.clone(),
            output_path: library.output_path.clone(),
            span: library.span,
            classes,
            enums,
        },
        diagnostics,
    }
}

fn lower_enum(e: &dust_resolver::ResolvedEnum) -> LoweringOutcome<EnumIr> {
    let mut diagnostics: Vec<Diagnostic> = Vec::new();
    let serde = lower_class_serde_config(&e.name, &e.configs, &mut diagnostics);
    let variants: Vec<EnumVariantIr> = e
        .variants
        .iter()
        .map(|v| EnumVariantIr {
            name: v.name.clone(),
            span: v.span,
        })
        .collect();
    LoweringOutcome {
        value: EnumIr {
            name: e.name.clone(),
            span: e.span,
            variants,
            traits: e.traits.clone(),
            serde,
        },
        diagnostics,
    }
}

fn lower_class(class: &ResolvedClass) -> LoweringOutcome<ClassIr> {
    let mut diagnostics = Vec::new();
    let serde = lower_class_serde_config(&class.name, &class.configs, &mut diagnostics);

    let fields = class
        .fields
        .iter()
        .map(|field| {
            let outcome = lower_type(field.type_source.as_deref());
            diagnostics.extend(outcome.diagnostics);
            FieldIr {
                name: field.name.clone(),
                ty: outcome.value,
                span: field.span,
                has_default: field.has_default,
                serde: lower_field_serde_config(&field.name, &field.configs, &mut diagnostics),
            }
        })
        .collect::<Vec<_>>();

    let constructors = class
        .constructors
        .iter()
        .map(|constructor| {
            let params = constructor
                .params
                .iter()
                .map(|param| {
                    let outcome = param
                        .type_source
                        .as_deref()
                        .map(|source| lower_type(Some(source)))
                        .unwrap_or_else(|| infer_param_type(param.name.as_str(), &fields));
                    diagnostics.extend(outcome.diagnostics);
                    ConstructorParamIr {
                        name: param.name.clone(),
                        ty: outcome.value,
                        span: SpanIr::new(class.span.file_id, param.span),
                        kind: match param.kind {
                            ParameterKind::Positional => ParamKind::Positional,
                            ParameterKind::Named => ParamKind::Named,
                        },
                        has_default: param.has_default,
                    }
                })
                .collect();

            ConstructorIr {
                name: constructor.name.clone(),
                span: SpanIr::new(class.span.file_id, constructor.span),
                params,
            }
        })
        .collect();

    LoweringOutcome {
        value: ClassIr {
            kind: class.kind,
            name: class.name.clone(),
            is_abstract: class.is_abstract,
            superclass_name: class.superclass_name.clone(),
            span: class.span,
            fields,
            constructors,
            traits: class.traits.clone(),
            serde,
        },
        diagnostics,
    }
}

fn infer_param_type(name: &str, fields: &[FieldIr]) -> LoweringOutcome<TypeIr> {
    if let Some(field) = fields.iter().find(|field| field.name == name) {
        LoweringOutcome::new(field.ty.clone())
    } else {
        LoweringOutcome::new(TypeIr::unknown())
    }
}

fn lower_type(source: Option<&str>) -> LoweringOutcome<TypeIr> {
    let Some(source) = source.map(str::trim).filter(|source| !source.is_empty()) else {
        return LoweringOutcome::new(TypeIr::unknown());
    };

    LoweringOutcome::new(parse_type(source))
}

fn lower_class_serde_config(
    class_name: &str,
    configs: &[ConfigApplicationIr],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<SerdeClassConfigIr> {
    let mut serde = SerdeClassConfigIr::default();
    let mut saw_serde = false;

    for config in configs {
        if !is_serde_config(config) {
            continue;
        }
        saw_serde = true;

        for (key, value) in parse_serde_arguments(config.arguments_source.as_deref(), diagnostics) {
            match key {
                "rename" => match parse_string_literal(value) {
                    Some(rename) => serde.rename = Some(rename),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-string `SerDe(rename: ...)` value"
                    ))),
                },
                "renameAll" => match parse_serde_rename_rule(value) {
                    Some(rule) => serde.rename_all = Some(rule),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses an unknown `SerDe(renameAll: ...)` rule"
                    ))),
                },
                "disallowUnrecognizedKeys" => match parse_bool_literal(value) {
                    Some(flag) => serde.disallow_unrecognized_keys = flag,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` uses a non-boolean `SerDe(disallowUnrecognizedKeys: ...)` value"
                    ))),
                },
                "aliases"
                | "defaultValue"
                | "skip"
                | "skipSerializing"
                | "skipDeserializing"
                | "using" => {
                    diagnostics.push(Diagnostic::error(format!(
                        "class `{class_name}` does not support `SerDe({key}: ...)`"
                    )));
                }
                unknown => diagnostics.push(Diagnostic::warning(format!(
                    "class `{class_name}` uses unknown `SerDe` option `{unknown}`"
                ))),
            }
        }
    }

    saw_serde.then_some(serde)
}

fn lower_field_serde_config(
    field_name: &str,
    configs: &[ConfigApplicationIr],
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<SerdeFieldConfigIr> {
    let mut serde = SerdeFieldConfigIr::default();
    let mut saw_serde = false;

    for config in configs {
        if !is_serde_config(config) {
            continue;
        }
        saw_serde = true;

        for (key, value) in parse_serde_arguments(config.arguments_source.as_deref(), diagnostics) {
            match key {
                "rename" => match parse_string_literal(value) {
                    Some(rename) => serde.rename = Some(rename),
                    None => diagnostics.push(Diagnostic::error(format!(
                        "field `{field_name}` uses a non-string `SerDe(rename: ...)` value"
                    ))),
                },
                "aliases" => match parse_string_list(value) {
                    Some(aliases) => serde.aliases = aliases,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "field `{field_name}` uses a non-string-list `SerDe(aliases: ...)` value"
                    ))),
                },
                "using" => {
                    if let Some(codec_source) = parse_codec_source(field_name, value, diagnostics) {
                        serde.codec_source = Some(codec_source);
                    }
                }
                "defaultValue" => serde.default_value_source = Some(value.trim().to_owned()),
                "skip" => match parse_bool_literal(value) {
                    Some(true) => {
                        serde.skip_serializing = true;
                        serde.skip_deserializing = true;
                    }
                    Some(false) => {}
                    None => diagnostics.push(Diagnostic::error(format!(
                        "field `{field_name}` uses a non-boolean `SerDe(skip: ...)` value"
                    ))),
                },
                "skipSerializing" => match parse_bool_literal(value) {
                    Some(flag) => serde.skip_serializing = flag,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "field `{field_name}` uses a non-boolean `SerDe(skipSerializing: ...)` value"
                    ))),
                },
                "skipDeserializing" => match parse_bool_literal(value) {
                    Some(flag) => serde.skip_deserializing = flag,
                    None => diagnostics.push(Diagnostic::error(format!(
                        "field `{field_name}` uses a non-boolean `SerDe(skipDeserializing: ...)` value"
                    ))),
                },
                "renameAll" | "disallowUnrecognizedKeys" => diagnostics.push(
                    Diagnostic::error(format!(
                        "field `{field_name}` does not support `SerDe({key}: ...)`"
                    )),
                ),
                unknown => diagnostics.push(Diagnostic::warning(format!(
                    "field `{field_name}` uses unknown `SerDe` option `{unknown}`"
                ))),
            }
        }
    }

    saw_serde.then_some(serde)
}

fn is_serde_config(config: &ConfigApplicationIr) -> bool {
    config.symbol.0 == "derive_serde_annotation::SerDe"
}

fn parse_serde_arguments<'a>(
    source: Option<&'a str>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<(&'a str, &'a str)> {
    let Some(source) = source.map(str::trim).filter(|source| !source.is_empty()) else {
        return Vec::new();
    };

    let Some(inner) = source
        .strip_prefix('(')
        .and_then(|inner| inner.strip_suffix(')'))
    else {
        diagnostics.push(Diagnostic::error(
            "SerDe config arguments must use parenthesized named arguments",
        ));
        return Vec::new();
    };

    let inner = inner.trim();
    if inner.is_empty() {
        return Vec::new();
    }

    let mut arguments = Vec::new();
    for item in split_top_level_items(inner) {
        if let Some((key, value)) = split_named_argument(item) {
            arguments.push((key.trim(), value.trim()));
        } else {
            diagnostics.push(Diagnostic::error(format!(
                "could not parse `SerDe` argument `{item}`"
            )));
        }
    }

    arguments
}

fn split_named_argument(source: &str) -> Option<(&str, &str)> {
    let mut depth_angle = 0_u32;
    let mut depth_paren = 0_u32;
    let mut depth_brace = 0_u32;
    let mut depth_bracket = 0_u32;
    let mut quote = None;
    let mut escape = false;

    for (index, ch) in source.char_indices() {
        if let Some(active_quote) = quote {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '\'' | '"' => quote = Some(ch),
            '<' => depth_angle += 1,
            '>' => depth_angle = depth_angle.saturating_sub(1),
            '(' => depth_paren += 1,
            ')' => depth_paren = depth_paren.saturating_sub(1),
            '{' => depth_brace += 1,
            '}' => depth_brace = depth_brace.saturating_sub(1),
            '[' => depth_bracket += 1,
            ']' => depth_bracket = depth_bracket.saturating_sub(1),
            ':' if depth_angle == 0
                && depth_paren == 0
                && depth_brace == 0
                && depth_bracket == 0 =>
            {
                return Some((&source[..index], &source[index + 1..]));
            }
            _ => {}
        }
    }

    None
}

fn split_top_level_items(source: &str) -> Vec<&str> {
    let mut items = Vec::new();
    let mut depth_angle = 0_u32;
    let mut depth_paren = 0_u32;
    let mut depth_brace = 0_u32;
    let mut depth_bracket = 0_u32;
    let mut quote = None;
    let mut escape = false;
    let mut start = 0_usize;

    for (index, ch) in source.char_indices() {
        if let Some(active_quote) = quote {
            if escape {
                escape = false;
                continue;
            }
            if ch == '\\' {
                escape = true;
                continue;
            }
            if ch == active_quote {
                quote = None;
            }
            continue;
        }

        match ch {
            '\'' | '"' => quote = Some(ch),
            '<' => depth_angle += 1,
            '>' => depth_angle = depth_angle.saturating_sub(1),
            '(' => depth_paren += 1,
            ')' => depth_paren = depth_paren.saturating_sub(1),
            '{' => depth_brace += 1,
            '}' => depth_brace = depth_brace.saturating_sub(1),
            '[' => depth_bracket += 1,
            ']' => depth_bracket = depth_bracket.saturating_sub(1),
            ',' if depth_angle == 0
                && depth_paren == 0
                && depth_brace == 0
                && depth_bracket == 0 =>
            {
                items.push(source[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }

    let tail = source[start..].trim();
    if !tail.is_empty() {
        items.push(tail);
    }

    items
}

fn parse_string_literal(source: &str) -> Option<String> {
    let source = source.trim();
    let first = source.chars().next()?;
    let last = source.chars().next_back()?;
    if source.len() < 2 || first != last || !matches!(first, '\'' | '"') {
        return None;
    }

    Some(source[1..source.len() - 1].to_owned())
}

fn parse_bool_literal(source: &str) -> Option<bool> {
    match source.trim() {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

fn parse_string_list(source: &str) -> Option<Vec<String>> {
    let source = source.trim();
    let inner = source.strip_prefix('[')?.strip_suffix(']')?.trim();
    if inner.is_empty() {
        return Some(Vec::new());
    }

    split_top_level_items(inner)
        .into_iter()
        .map(parse_string_literal)
        .collect()
}

fn parse_codec_source(
    field_name: &str,
    source: &str,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<String> {
    let source = source.trim();
    if source.is_empty() {
        diagnostics.push(
            Diagnostic::error(format!(
                "field `{field_name}` uses empty `SerDe(using: ...)` value"
            ))
            .with_note(codec_source_guidance()),
        );
        return None;
    }

    if parse_string_literal(source).is_some()
        || parse_bool_literal(source).is_some()
        || source == "null"
        || looks_like_number_literal(source)
        || looks_like_collection_literal(source)
        || looks_like_function_literal(source)
    {
        diagnostics.push(
            Diagnostic::error(format!(
                "field `{field_name}` uses invalid `SerDe(using: ...)` value `{source}`"
            ))
            .with_note(codec_source_guidance()),
        );
        return None;
    }

    if looks_like_bare_type_reference(source) {
        diagnostics.push(
            Diagnostic::error(format!(
                "field `{field_name}` uses suspicious `SerDe(using: ...)` type reference `{source}`"
            ))
            .with_note(codec_source_guidance()),
        );
        return None;
    }

    Some(source.to_owned())
}

fn codec_source_guidance() -> &'static str {
    "Use a codec object such as `const UnixEpochDateTimeCodec()` or `unixEpochDateTimeCodec`."
}

fn looks_like_number_literal(source: &str) -> bool {
    let source = source.trim();
    let Some(first) = source.chars().next() else {
        return false;
    };

    first.is_ascii_digit()
        || ((first == '-' || first == '+')
            && source
                .chars()
                .nth(1)
                .is_some_and(|next| next.is_ascii_digit()))
}

fn looks_like_collection_literal(source: &str) -> bool {
    let source = source.trim();
    (source.starts_with('[') && source.ends_with(']'))
        || (source.starts_with('{') && source.ends_with('}'))
}

fn looks_like_function_literal(source: &str) -> bool {
    source.contains("=>")
}

fn looks_like_bare_type_reference(source: &str) -> bool {
    let source = source.trim();
    !source.contains('(')
        && !source.contains('.')
        && source
            .chars()
            .next()
            .is_some_and(|first| first.is_ascii_uppercase())
}

fn parse_serde_rename_rule(source: &str) -> Option<SerdeRenameRuleIr> {
    match source.trim().rsplit('.').next()? {
        "lowerCase" => Some(SerdeRenameRuleIr::LowerCase),
        "upperCase" => Some(SerdeRenameRuleIr::UpperCase),
        "pascalCase" => Some(SerdeRenameRuleIr::PascalCase),
        "camelCase" => Some(SerdeRenameRuleIr::CamelCase),
        "snakeCase" => Some(SerdeRenameRuleIr::SnakeCase),
        "screamingSnakeCase" => Some(SerdeRenameRuleIr::ScreamingSnakeCase),
        "kebabCase" => Some(SerdeRenameRuleIr::KebabCase),
        "screamingKebabCase" => Some(SerdeRenameRuleIr::ScreamingKebabCase),
        _ => None,
    }
}

fn parse_type(source: &str) -> TypeIr {
    let (base, nullable) = strip_nullable(source);
    let base = base.trim();

    let type_ir = if base == "dynamic" {
        TypeIr::dynamic()
    } else if looks_like_function_type(base) {
        TypeIr::function(base)
    } else if looks_like_record_type(base) {
        TypeIr::record(base)
    } else if let Some(builtin) = parse_builtin(base) {
        TypeIr::builtin(builtin)
    } else if let Some((name, args)) = split_generic(base) {
        TypeIr::generic(
            name.trim(),
            split_top_level_args(args)
                .into_iter()
                .map(|arg| parse_type(arg.trim()))
                .collect(),
        )
    } else {
        TypeIr::named(base)
    };

    if nullable {
        type_ir.nullable()
    } else {
        type_ir
    }
}

fn strip_nullable(source: &str) -> (&str, bool) {
    if let Some(stripped) = source.strip_suffix('?') {
        (stripped, true)
    } else {
        (source, false)
    }
}

fn parse_builtin(source: &str) -> Option<BuiltinType> {
    match source {
        "String" => Some(BuiltinType::String),
        "int" => Some(BuiltinType::Int),
        "bool" => Some(BuiltinType::Bool),
        "double" => Some(BuiltinType::Double),
        "num" => Some(BuiltinType::Num),
        "Object" => Some(BuiltinType::Object),
        _ => None,
    }
}

fn split_generic(source: &str) -> Option<(&str, &str)> {
    let start = source.find('<')?;
    let end = source.rfind('>')?;
    if end <= start {
        return None;
    }
    Some((&source[..start], &source[start + 1..end]))
}

fn split_top_level_args(source: &str) -> Vec<&str> {
    let mut args = Vec::new();
    let mut depth_angle = 0_u32;
    let mut depth_paren = 0_u32;
    let mut depth_brace = 0_u32;
    let mut depth_bracket = 0_u32;
    let mut start = 0_usize;

    for (index, ch) in source.char_indices() {
        match ch {
            '<' => depth_angle += 1,
            '>' => depth_angle = depth_angle.saturating_sub(1),
            '(' => depth_paren += 1,
            ')' => depth_paren = depth_paren.saturating_sub(1),
            '{' => depth_brace += 1,
            '}' => depth_brace = depth_brace.saturating_sub(1),
            '[' => depth_bracket += 1,
            ']' => depth_bracket = depth_bracket.saturating_sub(1),
            ',' if depth_angle == 0
                && depth_paren == 0
                && depth_brace == 0
                && depth_bracket == 0 =>
            {
                args.push(source[start..index].trim());
                start = index + 1;
            }
            _ => {}
        }
    }

    let tail = source[start..].trim();
    if !tail.is_empty() {
        args.push(tail);
    }

    args
}

fn merged_fields_for_class(
    index: usize,
    classes: &[ClassIr],
    index_by_name: &HashMap<String, usize>,
    cache: &mut HashMap<usize, Vec<FieldIr>>,
    active_stack: &mut Vec<usize>,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<FieldIr> {
    if let Some(cached) = cache.get(&index) {
        return cached.clone();
    }

    if active_stack.contains(&index) {
        diagnostics.push(Diagnostic::error(format!(
            "cyclic superclass chain detected while lowering `{}`",
            classes[index].name
        )));
        return classes[index].fields.clone();
    }

    active_stack.push(index);

    let mut fields = if let Some(superclass_name) = classes[index].superclass_name.as_ref() {
        if let Some(super_index) = index_by_name.get(superclass_name) {
            merged_fields_for_class(
                *super_index,
                classes,
                index_by_name,
                cache,
                active_stack,
                diagnostics,
            )
        } else {
            Vec::new()
        }
    } else {
        Vec::new()
    };

    for field in &classes[index].fields {
        if let Some(existing) = fields
            .iter_mut()
            .find(|existing| existing.name == field.name)
        {
            *existing = field.clone();
        } else {
            fields.push(field.clone());
        }
    }

    active_stack.pop();
    cache.insert(index, fields.clone());
    fields
}

fn resolve_constructor_param_types(class: &mut ClassIr, diagnostics: &mut Vec<Diagnostic>) {
    for constructor in &mut class.constructors {
        for param in &mut constructor.params {
            if matches!(param.ty, TypeIr::Unknown) {
                if let Some(field) = class.fields.iter().find(|field| field.name == param.name) {
                    param.ty = field.ty.clone();
                } else {
                    diagnostics.push(Diagnostic::warning(format!(
                        "could not infer constructor parameter type for `{}`",
                        param.name
                    )));
                }
            }
        }
    }
}

fn looks_like_record_type(source: &str) -> bool {
    let Some(inner) = source
        .strip_prefix('(')
        .and_then(|inner| inner.strip_suffix(')'))
    else {
        return false;
    };

    let inner = inner.trim();
    if inner.is_empty() {
        return false;
    }

    inner.starts_with('{') || has_top_level_char(inner, ',')
}

fn looks_like_function_type(source: &str) -> bool {
    let mut depth_angle = 0_u32;
    let mut depth_paren = 0_u32;
    let mut depth_brace = 0_u32;
    let mut depth_bracket = 0_u32;

    for (index, ch) in source.char_indices() {
        match ch {
            '<' => depth_angle += 1,
            '>' => depth_angle = depth_angle.saturating_sub(1),
            '(' => depth_paren += 1,
            ')' => depth_paren = depth_paren.saturating_sub(1),
            '{' => depth_brace += 1,
            '}' => depth_brace = depth_brace.saturating_sub(1),
            '[' => depth_bracket += 1,
            ']' => depth_bracket = depth_bracket.saturating_sub(1),
            'F' if depth_angle == 0
                && depth_paren == 0
                && depth_brace == 0
                && depth_bracket == 0
                && index > 0 =>
            {
                let tail = &source[index..];
                if let Some(stripped) = tail.strip_prefix("Function") {
                    let prev = source[..index].chars().next_back().unwrap_or_default();
                    let after = stripped.trim_start();
                    if prev.is_whitespace() && after.starts_with('(') {
                        return true;
                    }
                }
            }
            _ => {}
        }
    }

    false
}

fn has_top_level_char(source: &str, target: char) -> bool {
    let mut depth_angle = 0_u32;
    let mut depth_paren = 0_u32;
    let mut depth_brace = 0_u32;
    let mut depth_bracket = 0_u32;

    for ch in source.chars() {
        match ch {
            '<' => depth_angle += 1,
            '>' => depth_angle = depth_angle.saturating_sub(1),
            '(' => depth_paren += 1,
            ')' => depth_paren = depth_paren.saturating_sub(1),
            '{' => depth_brace += 1,
            '}' => depth_brace = depth_brace.saturating_sub(1),
            '[' => depth_bracket += 1,
            ']' => depth_bracket = depth_bracket.saturating_sub(1),
            _ if ch == target
                && depth_angle == 0
                && depth_paren == 0
                && depth_brace == 0
                && depth_bracket == 0 =>
            {
                return true;
            }
            _ => {}
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::{lower_class, merged_fields_for_class, parse_type, split_top_level_args};
    use dust_ir::{
        BuiltinType, ClassIr, ClassKindIr, ConfigApplicationIr, FieldIr, SerdeRenameRuleIr, SpanIr,
        SymbolId, TraitApplicationIr, TypeIr,
    };
    use dust_resolver::{ResolvedClass, ResolvedField};
    use dust_text::{FileId, TextRange};

    fn span(start: u32, end: u32) -> SpanIr {
        SpanIr::new(FileId::new(99), TextRange::new(start, end))
    }

    #[test]
    fn parses_builtin_and_nullable_types() {
        let ty = parse_type("String?");
        assert!(ty.is_builtin(BuiltinType::String));
        assert!(ty.is_nullable());
    }

    #[test]
    fn parses_nested_generic_types() {
        let ty = parse_type("Map<String, List<int?>>");
        assert!(ty.is_named("Map"));
        assert_eq!(ty.args().len(), 2);
        assert!(ty.args()[1].is_named("List"));
        assert!(ty.args()[1].args()[0].is_builtin(BuiltinType::Int));
        assert!(ty.args()[1].args()[0].is_nullable());
    }

    #[test]
    fn keeps_function_like_types_as_named_fallbacks() {
        let ty = parse_type("void Function(String, int)?");
        assert!(ty.is_function());
        assert!(ty.is_nullable());
    }

    #[test]
    fn parses_record_types_without_falling_back_to_named() {
        let ty = parse_type("({String name, int age})?");
        assert!(ty.is_record());
        assert!(ty.is_nullable());
    }

    #[test]
    fn splits_top_level_args_without_breaking_nested_generics() {
        assert_eq!(
            split_top_level_args("String, Map<String, List<int>>, ({String name, int age})"),
            vec![
                "String",
                "Map<String, List<int>>",
                "({String name, int age})"
            ]
        );
    }

    #[test]
    fn merges_inherited_fields_before_own_fields() {
        let classes = vec![
            ClassIr {
                kind: ClassKindIr::Class,
                name: "Entity".to_owned(),
                is_abstract: true,
                superclass_name: None,
                span: span(0, 20),
                fields: vec![FieldIr {
                    name: "id".to_owned(),
                    ty: TypeIr::string(),
                    span: span(1, 2),
                    has_default: false,
                    serde: None,
                }],
                constructors: Vec::new(),
                traits: Vec::<TraitApplicationIr>::new(),
                serde: None,
            },
            ClassIr {
                kind: ClassKindIr::Class,
                name: "DetailedEntity".to_owned(),
                is_abstract: false,
                superclass_name: Some("Entity".to_owned()),
                span: span(20, 40),
                fields: vec![FieldIr {
                    name: "label".to_owned(),
                    ty: TypeIr::string(),
                    span: span(21, 22),
                    has_default: false,
                    serde: None,
                }],
                constructors: Vec::new(),
                traits: Vec::<TraitApplicationIr>::new(),
                serde: None,
            },
        ];
        let index_by_name = classes
            .iter()
            .enumerate()
            .map(|(index, class)| (class.name.clone(), index))
            .collect::<HashMap<_, _>>();
        let mut cache = HashMap::new();
        let mut active_stack = Vec::new();
        let mut diagnostics = Vec::new();

        let merged = merged_fields_for_class(
            1,
            &classes,
            &index_by_name,
            &mut cache,
            &mut active_stack,
            &mut diagnostics,
        );

        assert!(diagnostics.is_empty(), "{diagnostics:?}");
        assert_eq!(
            merged
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            vec!["id", "label"]
        );
    }

    #[test]
    fn lowers_serde_configs_into_ir() {
        let class = ResolvedClass {
            kind: ClassKindIr::Class,
            name: "User".to_owned(),
            is_abstract: false,
            superclass_name: None,
            span: span(0, 100),
            fields: vec![ResolvedField {
                name: "name".to_owned(),
                type_source: Some("String".to_owned()),
                has_default: false,
                span: span(20, 30),
                configs: vec![ConfigApplicationIr {
                    symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                    arguments_source: Some(
                        "(rename: 'full_name', aliases: ['fullName'], using: const NameCodec(), defaultValue: 'guest')"
                            .to_owned(),
                    ),
                    span: span(18, 30),
                }],
            }],
            constructors: Vec::new(),
            traits: Vec::new(),
            configs: vec![ConfigApplicationIr {
                symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                arguments_source: Some(
                    "(renameAll: SerDeRename.snakeCase, disallowUnrecognizedKeys: true)".to_owned(),
                ),
                span: span(1, 10),
            }],
        };

        let outcome = lower_class(&class);
        assert!(outcome.diagnostics.is_empty(), "{:?}", outcome.diagnostics);
        assert_eq!(
            outcome
                .value
                .serde
                .as_ref()
                .and_then(|serde| serde.rename_all),
            Some(SerdeRenameRuleIr::SnakeCase)
        );
        assert_eq!(
            outcome.value.fields[0]
                .serde
                .as_ref()
                .and_then(|serde| serde.rename.as_deref()),
            Some("full_name")
        );
        assert_eq!(
            outcome.value.fields[0]
                .serde
                .as_ref()
                .map(|serde| serde.aliases.clone()),
            Some(vec!["fullName".to_owned()])
        );
        assert_eq!(
            outcome.value.fields[0]
                .serde
                .as_ref()
                .and_then(|serde| serde.codec_source.as_deref()),
            Some("const NameCodec()")
        );
        assert_eq!(
            outcome.value.fields[0]
                .serde
                .as_ref()
                .and_then(|serde| serde.default_value_source.as_deref()),
            Some("'guest'")
        );
    }

    #[test]
    fn invalid_serde_options_produce_lowering_diagnostics() {
        let class = ResolvedClass {
            kind: ClassKindIr::Class,
            name: "User".to_owned(),
            is_abstract: false,
            superclass_name: None,
            span: span(0, 100),
            fields: vec![ResolvedField {
                name: "name".to_owned(),
                type_source: Some("String".to_owned()),
                has_default: false,
                span: span(20, 30),
                configs: vec![ConfigApplicationIr {
                    symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                    arguments_source: Some("(renameAll: SerDeRename.snakeCase)".to_owned()),
                    span: span(18, 30),
                }],
            }],
            constructors: Vec::new(),
            traits: Vec::new(),
            configs: vec![ConfigApplicationIr {
                symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                arguments_source: Some(
                    "(aliases: ['legacy'], using: const NameCodec())".to_owned(),
                ),
                span: span(1, 10),
            }],
        };

        let outcome = lower_class(&class);
        assert_eq!(outcome.diagnostics.len(), 3);
        assert!(outcome.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("class `User` does not support `SerDe(aliases: ...)`")
        }));
        assert!(outcome.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("class `User` does not support `SerDe(using: ...)`")
        }));
        assert!(outcome.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("field `name` does not support `SerDe(renameAll: ...)`")
        }));
    }

    #[test]
    fn invalid_serde_using_values_produce_lowering_diagnostics() {
        let class = ResolvedClass {
            kind: ClassKindIr::Class,
            name: "User".to_owned(),
            is_abstract: false,
            superclass_name: None,
            span: span(0, 100),
            fields: vec![
                ResolvedField {
                    name: "emptyCodec".to_owned(),
                    type_source: Some("DateTime".to_owned()),
                    has_default: false,
                    span: span(20, 30),
                    configs: vec![ConfigApplicationIr {
                        symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                        arguments_source: Some("(using: )".to_owned()),
                        span: span(18, 30),
                    }],
                },
                ResolvedField {
                    name: "stringCodec".to_owned(),
                    type_source: Some("DateTime".to_owned()),
                    has_default: false,
                    span: span(31, 40),
                    configs: vec![ConfigApplicationIr {
                        symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                        arguments_source: Some("(using: 'codec')".to_owned()),
                        span: span(31, 40),
                    }],
                },
                ResolvedField {
                    name: "nullCodec".to_owned(),
                    type_source: Some("DateTime".to_owned()),
                    has_default: false,
                    span: span(41, 50),
                    configs: vec![ConfigApplicationIr {
                        symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                        arguments_source: Some("(using: null)".to_owned()),
                        span: span(41, 50),
                    }],
                },
                ResolvedField {
                    name: "lambdaCodec".to_owned(),
                    type_source: Some("DateTime".to_owned()),
                    has_default: false,
                    span: span(51, 60),
                    configs: vec![ConfigApplicationIr {
                        symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                        arguments_source: Some("(using: () => const DateTimeCodec())".to_owned()),
                        span: span(51, 60),
                    }],
                },
                ResolvedField {
                    name: "typeCodec".to_owned(),
                    type_source: Some("DateTime".to_owned()),
                    has_default: false,
                    span: span(61, 70),
                    configs: vec![ConfigApplicationIr {
                        symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                        arguments_source: Some("(using: DateTimeCodec)".to_owned()),
                        span: span(61, 70),
                    }],
                },
                ResolvedField {
                    name: "validCodec".to_owned(),
                    type_source: Some("DateTime".to_owned()),
                    has_default: false,
                    span: span(71, 80),
                    configs: vec![ConfigApplicationIr {
                        symbol: SymbolId::new("derive_serde_annotation::SerDe"),
                        arguments_source: Some("(using: const DateTimeCodec())".to_owned()),
                        span: span(71, 80),
                    }],
                },
            ],
            constructors: Vec::new(),
            traits: Vec::new(),
            configs: Vec::new(),
        };

        let outcome = lower_class(&class);

        assert_eq!(outcome.diagnostics.len(), 5);
        assert!(outcome.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("field `emptyCodec` uses empty `SerDe(using: ...)` value")
        }));
        assert!(outcome.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("field `stringCodec` uses invalid `SerDe(using: ...)` value `'codec'`")
        }));
        assert!(outcome.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("field `nullCodec` uses invalid `SerDe(using: ...)` value `null`")
        }));
        assert!(outcome.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("field `lambdaCodec` uses invalid `SerDe(using: ...)` value `() => const DateTimeCodec()`")
        }));
        assert!(outcome.diagnostics.iter().any(|diagnostic| {
            diagnostic
                .message
                .contains("field `typeCodec` uses suspicious `SerDe(using: ...)` type reference `DateTimeCodec`")
        }));
        assert_eq!(
            outcome.value.fields[5]
                .serde
                .as_ref()
                .and_then(|serde| serde.codec_source.as_deref()),
            Some("const DateTimeCodec()")
        );
        for field in &outcome.value.fields[..5] {
            assert_eq!(
                field
                    .serde
                    .as_ref()
                    .and_then(|serde| serde.codec_source.as_deref()),
                None
            );
        }
        assert!(outcome.diagnostics.iter().all(|diagnostic| {
            diagnostic.notes.iter().any(|note| {
                note.contains("Use a codec object such as `const UnixEpochDateTimeCodec()`")
            })
        }));
    }
}
