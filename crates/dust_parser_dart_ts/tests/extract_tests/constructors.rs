use dust_parser_dart::{ParameterKind, ParsedClassKind, ParsedTypeKind};

use crate::support::parse;

/// Parameter annotations and redirecting factory constructor shapes.
#[path = "constructors/redirecting.rs"]
mod redirecting;

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
fn extracts_dotted_constructor_parameter_defaults() {
    let result = parse(
        3,
        r#"
enum ShoppingAccessLevel { staff }

class StaffDashboardScreen {
  const StaffDashboardScreen({
    this.access = ShoppingAccessLevel.staff,
  });

  final ShoppingAccessLevel access;
}
"#,
    );

    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let class = &result.library.classes[0];
    let access = &class.constructors[0].params[0];
    assert_eq!(access.name, "access");
    assert_eq!(
        access.default_value_source.as_deref(),
        Some("ShoppingAccessLevel.staff")
    );
}
