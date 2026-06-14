use dust_parser_dart::ParsedTypeKind;

use crate::support::parse;

#[test]
fn extracts_remaining_top_level_declaration_surfaces() {
    let result = parse(
        23,
        r#"
@Deprecated('mixin')
mixin Auditable {
  final String auditId;
}

@Deprecated('extension')
extension UserFormatting on User {
  String get label => 'user';
}

@Deprecated('extension-type')
extension type const UserId(String value) {}

@Deprecated('function')
String greet(User user, {String prefix = 'Hi'}) => '$prefix ${user.name}';

@Deprecated('variables')
final List<String> names = const ['a'];
const timeoutSeconds = 30;
external String host;

@Deprecated('typedef')
typedef UserMap = Map<String, User>;
"#,
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let library = &result.library;

    assert_eq!(library.mixins.len(), 1);
    assert_eq!(library.mixins[0].name, "Auditable");
    assert_eq!(library.mixins[0].annotations[0].name, "Deprecated");
    assert_eq!(library.mixins[0].fields.len(), 1);
    assert_eq!(library.mixins[0].fields[0].name, "auditId");
    assert_eq!(
        library.mixins[0].fields[0].type_source.as_deref(),
        Some("String")
    );

    assert_eq!(library.extensions.len(), 1);
    assert_eq!(
        library.extensions[0].name.as_deref(),
        Some("UserFormatting")
    );
    assert_eq!(
        library.extensions[0].on_type_source.as_deref(),
        Some("User")
    );

    assert_eq!(library.extension_types.len(), 1);
    assert_eq!(library.extension_types[0].name, "UserId");
    assert_eq!(library.extension_types[0].representation_name, "value");
    assert_eq!(
        library.extension_types[0]
            .representation_type_source
            .as_deref(),
        Some("String")
    );

    assert_eq!(library.functions.len(), 1);
    assert_eq!(library.functions[0].name, "greet");
    assert_eq!(
        library.functions[0].return_type_source.as_deref(),
        Some("String")
    );
    assert_eq!(library.functions[0].params.len(), 2);
    assert_eq!(library.functions[0].params[1].name, "prefix");
    assert_eq!(
        library.functions[0].params[1]
            .default_value_source
            .as_deref(),
        Some("'Hi'")
    );

    assert_eq!(library.variables.len(), 3);
    assert_eq!(library.variables[0].name, "names");
    assert_eq!(
        library.variables[0].type_source.as_deref(),
        Some("List<String>")
    );
    assert_eq!(
        library.variables[0].initializer_source.as_deref(),
        Some("const ['a']")
    );
    assert!(library.variables[0].initializer_span.is_some());
    assert_eq!(library.variables[1].name, "timeoutSeconds");
    assert_eq!(
        library.variables[1].initializer_source.as_deref(),
        Some("30")
    );
    assert_eq!(library.variables[2].name, "host");
    assert_eq!(library.variables[2].type_source.as_deref(), Some("String"));
    assert_eq!(library.variables[2].initializer_source, None);

    assert_eq!(library.typedefs.len(), 1);
    assert_eq!(library.typedefs[0].name, "UserMap");
    assert_eq!(
        library.typedefs[0].aliased_type_source.as_deref(),
        Some("Map<String, User>")
    );
    assert!(matches!(
        library.typedefs[0]
            .parsed_aliased_type
            .as_ref()
            .map(|ty| &ty.kind),
        Some(ParsedTypeKind::Named { name, args }) if name == "Map" && args.len() == 2
    ));
}
