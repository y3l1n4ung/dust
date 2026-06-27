use std::collections::BTreeMap;

use dust_ir::{
    AnnotationIr, AnnotationNumberKindIr, AnnotationValueIr, ExprSourceIr, NameIr, SpanIr, SymbolId,
};
use dust_parser_dart::{
    ParsedAnnotation, ParsedAnnotationNumberKind, ParsedAnnotationValue,
    ParsedAnnotationValueRootKind,
};
use dust_text::{FileId, TextRange};

/// Converts one parsed annotation into semantic IR, optionally attaching a resolved symbol.
pub fn annotation_ir_from_parsed(
    file_id: FileId,
    annotation: &ParsedAnnotation,
    resolved_symbol: Option<SymbolId>,
) -> AnnotationIr {
    let name = name_ir(file_id, &annotation.qualified_name, annotation.span);
    let (positional_args, named_args) = annotation_argument_values(file_id, annotation);

    AnnotationIr {
        raw_name: name.source,
        short_name: name.short,
        prefix: name.prefix,
        positional_args,
        named_args,
        resolved_symbol,
        span: SpanIr::new(file_id, annotation.span),
    }
}

/// Converts parsed annotation arguments into semantic IR values.
pub(crate) fn annotation_argument_values(
    file_id: FileId,
    annotation: &ParsedAnnotation,
) -> (Vec<AnnotationValueIr>, BTreeMap<String, AnnotationValueIr>) {
    if let Some(arguments) = &annotation.parsed_arguments {
        let positional = arguments
            .positional
            .iter()
            .map(|argument| match &argument.value {
                Some(value) => annotation_value_ir(file_id, value),
                None => expression_value(file_id, argument.source.clone(), argument.span),
            })
            .collect();
        let named = arguments
            .named
            .iter()
            .map(|argument| {
                (
                    argument.name.clone(),
                    match &argument.value {
                        Some(value) => annotation_value_ir(file_id, value),
                        None => expression_value(
                            file_id,
                            argument.value_source.clone(),
                            argument.value_span,
                        ),
                    },
                )
            })
            .collect();
        return (positional, named);
    }

    let mut positional = Vec::new();
    let mut index = 0;
    while let Some(source) = annotation.positional_argument_source(index) {
        positional.push(expression_value(
            file_id,
            source.to_owned(),
            annotation.span,
        ));
        index += 1;
    }
    let named = annotation
        .named_arguments()
        .into_iter()
        .map(|(name, source)| {
            (
                name.to_owned(),
                expression_value(file_id, source.to_owned(), annotation.span),
            )
        })
        .collect();
    (positional, named)
}

/// Converts a parser-owned annotation value into semantic IR.
fn annotation_value_ir(file_id: FileId, value: &ParsedAnnotationValue) -> AnnotationValueIr {
    match &value.kind {
        ParsedAnnotationValueRootKind::Null => AnnotationValueIr::Null,
        ParsedAnnotationValueRootKind::Bool(flag) => AnnotationValueIr::Bool(*flag),
        ParsedAnnotationValueRootKind::String(source) => AnnotationValueIr::String(source.clone()),
        ParsedAnnotationValueRootKind::Number(kind) => AnnotationValueIr::Number {
            source: value.source.clone(),
            kind: match kind {
                ParsedAnnotationNumberKind::Int => AnnotationNumberKindIr::Int,
                ParsedAnnotationNumberKind::Double => AnnotationNumberKindIr::Double,
            },
        },
        ParsedAnnotationValueRootKind::List => AnnotationValueIr::List(Vec::new()),
        ParsedAnnotationValueRootKind::Set => AnnotationValueIr::Set(Vec::new()),
        ParsedAnnotationValueRootKind::Map => AnnotationValueIr::Map(Vec::new()),
        ParsedAnnotationValueRootKind::Record => AnnotationValueIr::Record(Vec::new()),
        ParsedAnnotationValueRootKind::Constructor { name } => AnnotationValueIr::Constructor {
            name: name_ir(file_id, name, value.span),
            positional_args: Vec::new(),
            named_args: BTreeMap::new(),
        },
        ParsedAnnotationValueRootKind::Member(name) => {
            AnnotationValueIr::Member(name_ir(file_id, name, value.span))
        }
        ParsedAnnotationValueRootKind::Expression => {
            expression_value(file_id, value.source.clone(), value.span)
        }
    }
}

/// Wraps a raw expression source as an annotation value.
fn expression_value(file_id: FileId, source: String, span: TextRange) -> AnnotationValueIr {
    AnnotationValueIr::Expression(ExprSourceIr {
        source,
        span: SpanIr::new(file_id, span),
    })
}

/// Builds normalized name IR from a parsed annotation name.
fn name_ir(file_id: FileId, source: &str, span: TextRange) -> NameIr {
    let source = source.trim().to_owned();
    let (prefix, short) = source
        .rsplit_once('.')
        .map(|(prefix, short)| (Some(prefix.to_owned()), short.to_owned()))
        .unwrap_or_else(|| (None, source.clone()));

    NameIr {
        source,
        short,
        prefix,
        span: SpanIr::new(file_id, span),
    }
}
