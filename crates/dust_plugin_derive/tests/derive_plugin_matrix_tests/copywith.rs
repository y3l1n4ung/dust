use dust_ir::{ParamKind, TypeIr};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_plugin_derive::register_plugin;

use super::support::{
    class, constructor, constructor_param, field, library, members_for_class, span,
};

#[test]
fn copywith_uses_named_arguments_without_braces_in_constructor_calls() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![class(
            "Request",
            vec![
                field("path", TypeIr::string()),
                field(
                    "headers",
                    TypeIr::map_of(TypeIr::string(), TypeIr::string()),
                ),
            ],
            vec![constructor(
                Some("create"),
                vec![
                    constructor_param("path", TypeIr::string(), ParamKind::Named),
                    constructor_param(
                        "headers",
                        TypeIr::map_of(TypeIr::string(), TypeIr::string()),
                        ParamKind::Named,
                    ),
                ],
            )],
            &["dust_dart::CopyWith"],
        )]),
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "Request");

    assert_eq!(contribution.mixin_members.len(), 1);
    assert_eq!(
        members,
        [r#"Request copyWith({
  String? path,
  Map<String, String>? headers,
}) {
  final self = this as Request;
  final nextHeaders = Map<String, String>.of(headers ?? self.headers);

  return Request.create(
    path: path ?? self.path,
    headers: nextHeaders,
  );
}"#
        .to_owned()]
        .as_slice()
    );
}

#[test]
fn copywith_renders_nested_generic_and_dynamic_casts() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![class(
            "Payload",
            vec![
                field("items", TypeIr::list_of(TypeIr::string()).nullable()),
                field("extra", TypeIr::dynamic()),
                field("transform", TypeIr::function("void Function(String, int)")),
                field("summary", TypeIr::record("(String, int)")),
            ],
            vec![constructor(
                None,
                vec![
                    constructor_param(
                        "items",
                        TypeIr::list_of(TypeIr::string()).nullable(),
                        ParamKind::Positional,
                    ),
                    constructor_param("extra", TypeIr::dynamic(), ParamKind::Positional),
                    constructor_param(
                        "transform",
                        TypeIr::function("void Function(String, int)"),
                        ParamKind::Positional,
                    ),
                    constructor_param(
                        "summary",
                        TypeIr::record("(String, int)"),
                        ParamKind::Positional,
                    ),
                ],
            )],
            &["dust_dart::CopyWith"],
        )]),
        &SymbolPlan::default(),
    );

    let members = members_for_class(&contribution, "Payload");
    assert_eq!(
        members,
        [r#"Payload copyWith({
  Option<List<String>?> items = const None(),
  Option<dynamic> extra = const None(),
  void Function(String, int)? transform,
  (String, int)? summary,
}) {
  final self = this as Payload;
  final nextItemsSource = switch (items) {
    None<List<String>?>() => self.items,
    Some<List<String>?>(:final value) => value,
  };
  final nextItems = nextItemsSource == null ? null : List<String>.of(nextItemsSource);
  final nextExtra = switch (extra) {
    None<dynamic>() => self.extra,
    Some<dynamic>(:final value) => value,
  };

  return Payload(
    nextItems,
    nextExtra,
    transform ?? self.transform,
    summary ?? self.summary,
  );
}"#
        .to_owned()]
        .as_slice()
    );
}

#[test]
fn copywith_uses_option_update_for_option_and_unknown_fields() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![class(
            "Profile",
            vec![
                field(
                    "nickname",
                    TypeIr::generic("Option", vec![TypeIr::string()]),
                ),
                field("metadata", TypeIr::unknown()),
            ],
            vec![constructor(
                None,
                vec![
                    constructor_param(
                        "nickname",
                        TypeIr::generic("Option", vec![TypeIr::string()]),
                        ParamKind::Named,
                    ),
                    constructor_param("metadata", TypeIr::unknown(), ParamKind::Named),
                ],
            )],
            &["dust_dart::CopyWith"],
        )]),
        &SymbolPlan::default(),
    );

    let members = members_for_class(&contribution, "Profile");
    assert_eq!(
        members,
        [r#"Profile copyWith({
  Option<Option<String>> nickname = const None(),
  Option<Object?> metadata = const None(),
}) {
  final self = this as Profile;
  final nextNickname = switch (nickname) {
    None<Option<String>>() => self.nickname,
    Some<Option<String>>(:final value) => value,
  };
  final nextMetadata = switch (metadata) {
    None<Object?>() => self.metadata,
    Some<Object?>(:final value) => value,
  };

  return Profile(
    nickname: nextNickname,
    metadata: nextMetadata,
  );
}"#
        .to_owned()]
        .as_slice()
    );
}

#[test]
fn copywith_uses_stable_temp_bindings_for_nested_types() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![super::support::class(
            "Complex",
            vec![
                field(
                    "left",
                    TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::named("Node")))
                        .nullable(),
                ),
                field(
                    "right",
                    TypeIr::generic("Set", vec![TypeIr::list_of(TypeIr::string())]),
                ),
            ],
            vec![super::support::constructor(
                None,
                vec![
                    constructor_param(
                        "left",
                        TypeIr::map_of(TypeIr::string(), TypeIr::list_of(TypeIr::named("Node")))
                            .nullable(),
                        ParamKind::Positional,
                    ),
                    constructor_param(
                        "right",
                        TypeIr::generic("Set", vec![TypeIr::list_of(TypeIr::string())]),
                        ParamKind::Positional,
                    ),
                ],
            )],
            &["dust_dart::CopyWith"],
        )]),
        &SymbolPlan::default(),
    );
    let members = members_for_class(&contribution, "Complex");
    assert_eq!(
        members,
        [r#"Complex copyWith({
  Option<Map<String, List<Node>>?> left = const None(),
  Set<List<String>>? right,
}) {
  final self = this as Complex;
  final nextLeftSource = switch (left) {
    None<Map<String, List<Node>>?>() => self.left,
    Some<Map<String, List<Node>>?>(:final value) => value,
  };
  final nextLeft = nextLeftSource == null ? null : Map<String, List<Node>>.fromEntries(
    nextLeftSource.entries.map(
      (entry_0) => MapEntry(entry_0.key, List<Node>.of(entry_0.value)),
    ),
  );
  final nextRight = Set<List<String>>.of(
    (right ?? self.right).map((item_1) => List<String>.of(item_1)),
  );

  return Complex(
    nextLeft,
    nextRight,
  );
}"#
        .to_owned()]
        .as_slice()
    );
}

#[test]
fn copywith_handles_named_model_collections_without_aliasing() {
    let plugin = register_plugin();
    let contribution = plugin.emit(
        &library(vec![
            class(
                "Node",
                vec![field("id", TypeIr::string())],
                vec![constructor(
                    None,
                    vec![constructor_param(
                        "id",
                        TypeIr::string(),
                        ParamKind::Positional,
                    )],
                )],
                &["dust_dart::CopyWith"],
            ),
            class(
                "Graph",
                vec![field("nodes", TypeIr::list_of(TypeIr::named("Node")))],
                vec![constructor(
                    None,
                    vec![constructor_param(
                        "nodes",
                        TypeIr::list_of(TypeIr::named("Node")),
                        ParamKind::Positional,
                    )],
                )],
                &["dust_dart::CopyWith"],
            ),
        ]),
        &SymbolPlan::default(),
    );

    let members = members_for_class(&contribution, "Graph");
    assert_eq!(
        members,
        [r#"Graph copyWith({
  List<Node>? nodes,
}) {
  final self = this as Graph;
  final nextNodes = List<Node>.of(
    (nodes ?? self.nodes).map((item_0) => item_0.copyWith()),
  );

  return Graph(
    nextNodes,
  );
}"#
        .to_owned()]
        .as_slice()
    );
    let _ = span(0, 0);
}
