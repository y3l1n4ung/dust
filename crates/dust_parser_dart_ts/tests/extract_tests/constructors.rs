use dust_parser_dart::{ParameterKind, ParsedClassKind};

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
            "    @Query('page') int? page,\n",
            "    @Header('x-trace-id') String? traceId,\n",
            "  });\n",
            "}\n",
        ),
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let class = &result.library.classes[0];
    let method = &class.methods[0];
    assert_eq!(method.params.len(), 3);
    assert_eq!(method.params[0].annotations[0].name, "Query");
    assert_eq!(method.params[0].type_source.as_deref(), Some("int?"));
    assert_eq!(method.params[1].annotations[0].name, "Query");
    assert_eq!(method.params[1].type_source.as_deref(), Some("int?"));
    assert_eq!(method.params[2].annotations[0].name, "Header");
    assert_eq!(method.params[2].type_source.as_deref(), Some("String?"));
}

#[test]
fn extracts_redirecting_factory_constructor_shapes() {
    let result = parse(
        8,
        r#"
part 'api.g.dart';

abstract interface class Api {
  factory Api(Dio dio, {String? baseUrl}) = _$Api;
}
"#,
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let class = &result.library.classes[0];
    assert!(class.is_abstract);
    assert!(class.is_interface);
    assert_eq!(class.constructors.len(), 1);
    assert!(class.constructors[0].is_factory);
    assert_eq!(
        class.constructors[0].redirected_target_name.as_deref(),
        Some("_$Api")
    );
    assert_eq!(class.constructors[0].params.len(), 2);
    assert_eq!(
        class.constructors[0].params[0].type_source.as_deref(),
        Some("Dio")
    );
    assert_eq!(
        class.constructors[0].params[1].type_source.as_deref(),
        Some("String?")
    );
}
