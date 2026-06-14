use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, DartFileIr, FieldIr, ParamKind,
    SpanIr, TypeIr,
};
use dust_text::{FileId, TextRange};

use super::*;

fn span() -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(0_u32, 1_u32))
}

fn library(classes: Vec<ClassIr>) -> DartFileIr {
    DartFileIr {
        package_root: String::new(),
        package_name: String::new(),
        source_path: "user.dart".to_owned(),
        output_path: "user.g.dart".to_owned(),
        imports: Vec::new(),
        library: None,
        library_annotations: Vec::new(),
        import_directives: Vec::new(),
        export_directives: Vec::new(),
        part_directives: Vec::new(),
        part_of: None,
        span: span(),
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

fn class(name: &str) -> ClassIr {
    ClassIr {
        kind: ClassKindIr::Class,
        name: name.to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: None,
        span: span(),
        fields: Vec::new(),
        constructors: Vec::new(),
        methods: Vec::new(),
        traits: Vec::new(),
        configs: Vec::new(),
        serde: None,
    }
}

#[test]
fn emits_basic_from_row_extension() {
    let mut class = class("UserRow");
    class.fields.push(FieldIr {
        name: "id".to_owned(),
        ty: TypeIr::int(),
        span: span(),
        has_default: false,
        serde: None,
        configs: Vec::new(),
    });
    class.constructors.push(ConstructorIr {
        name: None,
        is_factory: false,
        redirected_target_source: None,
        redirected_target_name: None,
        span: span(),
        params: vec![ConstructorParamIr {
            name: "id".to_owned(),
            ty: TypeIr::int(),
            span: span(),
            kind: ParamKind::Named,
            has_default: false,
            default_value_source: None,
        }],
    });
    let library = library(vec![class.clone()]);

    assert_eq!(
        render_from_row_extension(&library, &class, &Default::default()),
        r#"extension UserRowFromRow on UserRow {
  static UserRow fromRow(Row row) {
    return UserRow(id: row.read<int>('id'));
  }
}

final bool _$userRowFromRowRegistered = registerRowMapper<UserRow>(UserRowFromRow.fromRow);"#
    );
}

#[test]
fn emits_no_constructor_from_row_failure_body() {
    let mut class = class("NoCtorRow");
    class.fields.push(FieldIr {
        name: "id".to_owned(),
        ty: TypeIr::int(),
        span: span(),
        has_default: false,
        serde: None,
        configs: Vec::new(),
    });
    let library = library(vec![class.clone()]);

    assert_eq!(
        render_from_row_extension(&library, &class, &Default::default()),
        r#"extension NoCtorRowFromRow on NoCtorRow {
  static NoCtorRow fromRow(Row row) {
    throw StateError('No usable constructor found for NoCtorRow');
  }
}"#
    );
}
