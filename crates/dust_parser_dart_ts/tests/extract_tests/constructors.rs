use dust_parser_dart::{ParameterKind, ParsedClassKind, ParsedTypeKind};

use crate::support::parse;

#[test]
fn extracts_named_constructor_and_named_parameters() {
    let result = parse(
        2,
        r#"
part 'user_profile.g.dart';

class UserProfile<T> {
  final List<T> items;
  final int page;

  const UserProfile.named({required this.items, this.page = 1});
}
"#,
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let class = &result.library.classes[0];
    assert_eq!(class.kind, ParsedClassKind::Class);
    assert_eq!(class.name, "UserProfile");
    assert_eq!(class.fields.len(), 2);
    assert_eq!(class.fields[0].type_source.as_deref(), Some("List<T>"));
    assert_eq!(class.fields[1].type_source.as_deref(), Some("int"));
    assert_eq!(class.constructors.len(), 1);
    assert_eq!(class.constructors[0].name.as_deref(), Some("named"));
    assert_eq!(class.constructors[0].params.len(), 2);
    assert_eq!(class.constructors[0].params[0].name, "items");
    assert_eq!(class.constructors[0].params[0].kind, ParameterKind::Named);
    assert_eq!(class.constructors[0].params[1].name, "page");
    assert!(class.constructors[0].params[1].has_default);
    assert_eq!(
        class.constructors[0].params[1]
            .default_value_source
            .as_deref(),
        Some("1")
    );
}

#[test]
fn extracts_annotations_for_multiple_named_method_parameters() {
    let result = parse(
        8,
        concat!(
            "abstract interface class TodoApi {\n",
            "  Future<void> list({\n",
            "    @Query('userId') int? userId,\n",
            "    @Query('page') int? page = defaultPage,\n",
            "    @Header('x-trace-id') String? traceId,\n",
            "    @Query('sort') String sort = const SortOption('name,asc'),\n",
            "  });\n",
            "}\n",
        ),
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let class = &result.library.classes[0];
    let method = &class.methods[0];
    assert_eq!(method.return_type_source.as_deref(), Some("Future<void>"));
    assert!(matches!(
        method.parsed_return_type.as_ref().map(|ty| &ty.kind),
        Some(ParsedTypeKind::Named { name, args }) if name == "Future" && args.len() == 1
    ));
    assert_eq!(method.params.len(), 4);
    assert_eq!(method.params[0].annotations[0].name, "Query");
    assert_eq!(method.params[0].type_source.as_deref(), Some("int?"));
    assert!(method.params[0].parsed_type.as_ref().unwrap().nullable);
    assert_eq!(method.params[1].name, "page");
    assert_eq!(method.params[1].annotations[0].name, "Query");
    assert_eq!(method.params[1].type_source.as_deref(), Some("int?"));
    assert_eq!(
        method.params[1].default_value_source.as_deref(),
        Some("defaultPage")
    );
    assert_eq!(method.params[2].annotations[0].name, "Header");
    assert_eq!(method.params[2].type_source.as_deref(), Some("String?"));
    assert_eq!(method.params[3].name, "sort");
    assert_eq!(method.params[3].annotations[0].name, "Query");
    assert_eq!(method.params[3].type_source.as_deref(), Some("String"));
    assert_eq!(
        method.params[3].default_value_source.as_deref(),
        Some("const SortOption('name,asc')")
    );
}

#[test]
fn extracts_redirecting_factory_constructor_shapes() {
    let result = parse(
        8,
        r#"
part 'api.g.dart';

abstract interface class Api {
  factory Api(Dio dio, {String? baseUrl}) = _$Api;
  factory Api.internal(Dio dio) = _$Api.internal;
}
"#,
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let class = &result.library.classes[0];
    assert!(class.is_abstract);
    assert!(class.is_interface);
    assert_eq!(class.constructors.len(), 2);
    assert!(class.constructors[0].is_factory);
    assert_eq!(class.constructors[0].name, None);
    assert_eq!(
        class.constructors[0].redirected_target_source.as_deref(),
        Some("_$Api")
    );
    assert_eq!(
        class.constructors[0].redirected_target_name.as_deref(),
        Some("_$Api")
    );
    assert_eq!(class.constructors[0].params.len(), 2);
    assert_eq!(
        class.constructors[0].params[0].type_source.as_deref(),
        Some("Dio")
    );
    assert!(matches!(
        class.constructors[0].params[0]
            .parsed_type
            .as_ref()
            .map(|ty| &ty.kind),
        Some(ParsedTypeKind::Named { name, args }) if name == "Dio" && args.is_empty()
    ));
    assert_eq!(
        class.constructors[0].params[1].type_source.as_deref(),
        Some("String?")
    );
    assert!(
        class.constructors[0].params[1]
            .parsed_type
            .as_ref()
            .unwrap()
            .nullable
    );
    assert_eq!(class.constructors[1].name.as_deref(), Some("internal"));
    assert_eq!(
        class.constructors[1].redirected_target_source.as_deref(),
        Some("_$Api.internal")
    );
    assert_eq!(
        class.constructors[1].redirected_target_name.as_deref(),
        Some("_$Api.internal")
    );
}

#[test]
fn extracts_current_and_legacy_constructor_forms() {
    let result = parse(
        21,
        r#"
class Parent {
  const Parent({required this.id, this.enabled = false});
  final int id;
  final bool enabled;
}

final class Child extends Parent {
  const Child({required super.id, required this.title, super.enabled = true});
  Child.legacy(int id, [String title = defaultTitle]) : this(id: id, title: title);
  factory Child.fromJson(Map<String, Object?> json) {
    return Child(id: json['id'] as int, title: json['title'] as String);
  }

  final String title;
}

abstract final class ChildDao {
  const factory ChildDao(Executor db) = _$ChildDao;
  factory ChildDao.open(Executor db) = _$ChildDao.open;
}
"#,
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let child = &result.library.classes[1];
    let dao = &result.library.classes[2];
    let child_summaries: Vec<_> = child
        .constructors
        .iter()
        .map(|constructor| {
            (
                constructor.name.as_deref(),
                constructor.is_factory,
                constructor.redirected_target_name.as_deref(),
                constructor
                    .params
                    .iter()
                    .map(|param| {
                        (
                            param.name.as_str(),
                            param.type_source.as_deref(),
                            param.kind,
                            param.has_default,
                            param.default_value_source.as_deref(),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
        })
        .collect();

    assert_eq!(
        child_summaries,
        vec![
            (
                None,
                false,
                None,
                vec![
                    ("id", None, ParameterKind::Named, false, None),
                    ("title", None, ParameterKind::Named, false, None),
                    ("enabled", None, ParameterKind::Named, true, Some("true")),
                ],
            ),
            (
                Some("legacy"),
                false,
                None,
                vec![
                    ("id", Some("int"), ParameterKind::Positional, false, None),
                    (
                        "title",
                        Some("String"),
                        ParameterKind::Positional,
                        true,
                        Some("defaultTitle"),
                    ),
                ],
            ),
            (
                Some("fromJson"),
                true,
                None,
                vec![(
                    "json",
                    Some("Map<String, Object?>"),
                    ParameterKind::Positional,
                    false,
                    None,
                )],
            ),
        ]
    );
    assert_eq!(dao.constructors.len(), 2);
    assert_eq!(dao.constructors[0].name, None);
    assert!(dao.constructors[0].is_factory);
    assert_eq!(
        dao.constructors[0].redirected_target_name.as_deref(),
        Some("_$ChildDao")
    );
    assert_eq!(dao.constructors[1].name.as_deref(), Some("open"));
    assert_eq!(
        dao.constructors[1].redirected_target_source.as_deref(),
        Some("_$ChildDao.open")
    );
    assert_eq!(
        dao.constructors[1].redirected_target_name.as_deref(),
        Some("_$ChildDao.open")
    );
}
