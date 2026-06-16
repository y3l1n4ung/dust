use dust_ir::{
    ClassIr, ClassKindIr, ConstructorIr, ConstructorParamIr, FieldIr, LibraryIr, ParamKind,
    SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use crate::support::{members_for_class, span};

#[test]
fn copywith_copies_collection_fields() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &LibraryIr {
            package_root: ".".to_owned(),
            package_name: "dust_test".to_owned(),
            source_path: "lib/catalog.dart".to_owned(),
            output_path: "lib/catalog.g.dart".to_owned(),
            imports: Vec::new(),
            library: None,
            library_annotations: Vec::new(),
            import_directives: Vec::new(),
            export_directives: Vec::new(),
            part_directives: Vec::new(),
            part_of: None,
            span: span(0, 100),
            classes: vec![ClassIr {
                kind: ClassKindIr::Class,
                name: "Catalog".to_owned(),
                is_abstract: false,
                is_interface: false,
                superclass_name: None,
                span: span(10, 80),
                fields: vec![
                    FieldIr {
                        name: "groups".to_owned(),
                        ty: TypeIr::list_of(TypeIr::list_of(TypeIr::string())),
                        span: span(18, 19),
                        has_default: false,
                        serde: None,
                        configs: Vec::new(),
                    },
                    FieldIr {
                        name: "metrics".to_owned(),
                        ty: TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())),
                        span: span(41, 50),
                        has_default: false,
                        serde: None,
                        configs: Vec::new(),
                    },
                ],
                constructors: vec![ConstructorIr {
                    name: None,
                    is_factory: false,
                    redirected_target_source: None,
                    redirected_target_name: None,
                    span: span(40, 60),
                    params: vec![
                        ConstructorParamIr {
                            name: "groups".to_owned(),
                            ty: TypeIr::list_of(TypeIr::list_of(TypeIr::string())),
                            span: span(40, 41),
                            kind: ParamKind::Named,
                            has_default: false,
                            default_value_source: None,
                        },
                        ConstructorParamIr {
                            name: "metrics".to_owned(),
                            ty: TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::int())),
                            span: span(48, 49),
                            kind: ParamKind::Named,
                            has_default: false,
                            default_value_source: None,
                        },
                    ],
                }],
                methods: Vec::new(),
                traits: vec![TraitApplicationIr {
                    symbol: SymbolId::new("dust_dart::CopyWith"),
                    span: span(5, 9),
                }],
                configs: Vec::new(),
                serde: None,
            }],
            mixins: Vec::new(),
            extensions: Vec::new(),
            extension_types: Vec::new(),
            functions: Vec::new(),
            variables: Vec::new(),
            typedefs: Vec::new(),
            enums: Vec::new(),
            query_calls: Vec::new(),
        },
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "Catalog");

    assert_eq!(members.len(), 1);
    assert_eq!(
        members,
        [r#"Catalog copyWith({
  List<List<String>>? groups,
  Map<String, List<int>>? metrics,
}) {
  final self = this as Catalog;
  final nextGroups = List<List<String>>.of(
    (groups ?? self.groups).map((item_0) => List<String>.of(item_0)),
  );
  final nextMetrics = Map<String, List<int>>.fromEntries(
    (metrics ?? self.metrics).entries.map(
      (entry_1) => MapEntry(entry_1.key, List<int>.of(entry_1.value)),
    ),
  );

  return Catalog(
    groups: nextGroups,
    metrics: nextMetrics,
  );
}"#
        .to_owned()]
        .as_slice()
    );
}
