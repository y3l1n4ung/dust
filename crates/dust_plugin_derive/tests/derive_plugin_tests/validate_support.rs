use std::collections::BTreeMap;

use dust_ir::{
    AnnotationNumberKindIr, AnnotationValueIr, ClassIr, ClassKindIr, ConfigApplicationIr,
    ConstructorIr, ExprSourceIr, FieldIr, LibraryIr, NameIr, SymbolId, TraitApplicationIr, TypeIr,
};
use dust_parser_dart::{AnnotationValue, parse_annotation_named_values};

use crate::support::span;

pub(super) fn library(classes: Vec<ClassIr>) -> LibraryIr {
    LibraryIr {
        package_root: ".".to_owned(),
        package_name: "dust_test".to_owned(),
        source_path: "lib/model.dart".to_owned(),
        output_path: "lib/model.g.dart".to_owned(),
        imports: Vec::new(),
        library: None,
        library_annotations: Vec::new(),
        import_directives: Vec::new(),
        export_directives: Vec::new(),
        part_directives: Vec::new(),
        part_of: None,
        span: span(0, 100),
        classes,
        mixins: Vec::new(),
        extensions: Vec::new(),
        extension_types: Vec::new(),
        functions: Vec::new(),
        variables: Vec::new(),
        typedefs: Vec::new(),
        enums: Vec::new(),
        query_calls: Vec::new(),
    }
}

pub(super) fn class(name: &str) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: None,
        span: span(10, 80),
        fields: Vec::new(),
        constructors: vec![ConstructorIr {
            name: None,
            is_factory: false,
            redirected_target_source: None,
            redirected_target_name: None,
            span: span(40, 60),
            params: Vec::new(),
        }],
        methods: Vec::new(),
        traits: vec![TraitApplicationIr {
            symbol: SymbolId::new("dust_dart::Validate"),
            span: span(5, 9),
        }],
        configs: Vec::new(),
        serde: None,
    }
}

pub(super) fn field(name: &str, ty: TypeIr, configs: Vec<ConfigApplicationIr>) -> FieldIr {
    FieldIr {
        name: name.to_owned(),
        ty,
        span: span(20, 30),
        has_default: false,
        serde: None,
        configs,
    }
}

pub(super) fn validate(arguments: &str) -> ConfigApplicationIr {
    let named_args = parse_annotation_named_values(arguments)
        .expect("validate fixture arguments should parse")
        .into_iter()
        .map(|(name, value)| (name, annotation_value_ir(value)))
        .collect();
    ConfigApplicationIr::with_arguments(
        SymbolId::new("dust_dart::Validate"),
        Some(arguments.to_owned()),
        Vec::new(),
        named_args,
        span(20, 30),
    )
}

fn annotation_value_ir(value: AnnotationValue) -> AnnotationValueIr {
    match value {
        AnnotationValue::Bool(value) => AnnotationValueIr::Bool(value),
        AnnotationValue::String(value) => AnnotationValueIr::String(value),
        AnnotationValue::Number(source) => AnnotationValueIr::Number {
            kind: if source.contains('.') {
                AnnotationNumberKindIr::Double
            } else {
                AnnotationNumberKindIr::Int
            },
            source,
        },
        AnnotationValue::List(values) => {
            AnnotationValueIr::List(values.into_iter().map(annotation_value_ir).collect())
        }
        AnnotationValue::Record(values) => AnnotationValueIr::Record(
            values
                .into_iter()
                .map(|(name, value)| (name, annotation_value_ir(value)))
                .collect(),
        ),
        AnnotationValue::Constructor { name, named } => AnnotationValueIr::Constructor {
            name: name_ir(name),
            positional_args: Vec::new(),
            named_args: named
                .into_iter()
                .map(|(name, value)| (name, annotation_value_ir(value)))
                .collect::<BTreeMap<_, _>>(),
        },
        AnnotationValue::Member(name) => AnnotationValueIr::Member(name_ir(name)),
        AnnotationValue::Expression(source) => AnnotationValueIr::Expression(ExprSourceIr {
            source,
            span: span(20, 30),
        }),
    }
}

fn name_ir(source: String) -> NameIr {
    let (prefix, short) = source
        .rsplit_once('.')
        .map(|(prefix, short)| (Some(prefix.to_owned()), short.to_owned()))
        .unwrap_or_else(|| (None, source.clone()));
    NameIr {
        source,
        short,
        prefix,
        span: span(20, 30),
    }
}
