use dust_ir::{
    AnnotationValueIr, ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, EnumIr, FieldIr,
    LibraryIr, ParamKind, SerdeFieldConfigIr, SpanIr, SymbolId, TraitApplicationIr, TypeIr,
};
use dust_text::{FileId, TextRange};

/// Builds a source span for validation fixture IR.
pub(crate) fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(11), TextRange::new(start, end))
}

/// Builds a trait application fixture.
pub(crate) fn trait_application(symbol: &str) -> TraitApplicationIr {
    TraitApplicationIr {
        symbol: SymbolId::new(symbol),
        span: span(1, 5),
    }
}

/// Builds a field fixture with default serde settings.
pub(crate) fn field(name: &str, ty: TypeIr) -> FieldIr {
    FieldIr {
        name: name.to_owned(),
        ty,
        span: span(10, 20),
        has_default: false,
        serde: None,
        configs: Vec::new(),
    }
}

/// Builds a field fixture with a typed serde default value.
pub(crate) fn field_with_default(
    name: &str,
    ty: TypeIr,
    source: &str,
    value: AnnotationValueIr,
) -> FieldIr {
    FieldIr {
        name: name.to_owned(),
        ty,
        span: span(10, 20),
        has_default: false,
        serde: Some(SerdeFieldConfigIr {
            default_value_source: Some(source.to_owned()),
            default_value: Some(value),
            ..Default::default()
        }),
        configs: Vec::new(),
    }
}

/// Builds a constructor parameter fixture.
pub(crate) fn constructor_param(name: &str, ty: TypeIr, kind: ParamKind) -> ConstructorParamIr {
    ConstructorParamIr {
        name: name.to_owned(),
        ty,
        span: span(30, 35),
        kind,
        has_default: false,
        default_value_source: None,
    }
}

/// Builds a generative constructor fixture.
pub(crate) fn constructor(name: Option<&str>, params: Vec<ConstructorParamIr>) -> ConstructorIr {
    ConstructorIr {
        name: name.map(str::to_owned),
        is_factory: false,
        redirected_target_source: None,
        redirected_target_name: None,
        span: span(25, 60),
        params,
    }
}

/// Builds a class fixture with serde traits.
pub(crate) fn class(
    name: &str,
    fields: Vec<FieldIr>,
    constructors: Vec<ConstructorIr>,
    traits: &[&str],
) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: None,
        span: span(0, 100),
        fields,
        constructors,
        methods: Vec::new(),
        traits: traits
            .iter()
            .map(|symbol| trait_application(symbol))
            .collect(),
        configs: Vec::new(),
        serde: None,
    }
}

/// Builds a library fixture for validation tests.
pub(crate) fn library(classes: Vec<ClassIr>, enums: Vec<EnumIr>) -> LibraryIr {
    LibraryIr {
        package_root: ".".to_owned(),
        package_name: "dust_test".to_owned(),
        source_path: "lib/models.dart".to_owned(),
        output_path: "lib/models.g.dart".to_owned(),
        imports: Vec::new(),
        library: None,
        library_annotations: Vec::new(),
        import_directives: Vec::new(),
        export_directives: Vec::new(),
        part_directives: Vec::new(),
        part_of: None,
        span: span(0, 200),
        classes,
        mixins: Vec::new(),
        extensions: Vec::new(),
        extension_types: Vec::new(),
        functions: Vec::new(),
        variables: Vec::new(),
        typedefs: Vec::new(),
        enums,
        query_calls: Vec::new(),
    }
}
