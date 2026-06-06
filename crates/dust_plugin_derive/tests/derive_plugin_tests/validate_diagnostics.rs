use dust_ir::TypeIr;
use dust_plugin_api::DustPlugin;
use dust_plugin_derive::register_plugin;

use crate::validate_support::{class, field, library, validate};

#[test]
fn validates_bad_validate_field_configuration() {
    let plugin = register_plugin();
    let mut class = class("Broken");
    class.fields = vec![
        field("age", TypeIr::int(), vec![validate("(email: true)")]),
        field(
            "name",
            TypeIr::string(),
            vec![validate("(range: Range(min: 1))")],
        ),
        field(
            "code",
            TypeIr::string(),
            vec![validate("(length: UnknownLength.bad)")],
        ),
        field(
            "confirm",
            TypeIr::string(),
            vec![validate("(mustMatch: 'missing')")],
        ),
        field(
            "custom",
            TypeIr::string(),
            vec![validate("(custom: 'bad')")],
        ),
    ];

    let diagnostics = plugin.validate(&library(vec![class]));
    let messages = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        messages,
        vec![
            "`@Validate` string validators on `age` require `String` or `String?`",
            "`@Validate(range: ...)` on `name` requires int, double, or num",
            "`@Validate(mustMatch: 'missing')` references a missing field",
            "`@Validate(length: ...)` expects `Length(...)`",
            "`@Validate(custom: ...)` expects a function reference",
        ]
    );
}

#[test]
fn validates_bad_constructor_shapes() {
    let plugin = register_plugin();
    let mut class = class("BrokenRecords");
    class.fields = vec![
        field(
            "empty",
            TypeIr::string(),
            vec![validate("(length: Length())")],
        ),
        field(
            "doubleLength",
            TypeIr::string(),
            vec![validate("(length: Length(min: 1.5))")],
        ),
        field(
            "badLengthKey",
            TypeIr::string(),
            vec![validate("(length: Length(low: 1))")],
        ),
        field(
            "badRange",
            TypeIr::int(),
            vec![validate("(range: Range(min: 9, max: 1))")],
        ),
    ];

    let diagnostics = plugin.validate(&library(vec![class]));
    let messages = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        messages,
        vec![
            "`@Validate(length: ...)` constructor cannot be empty",
            "`@Validate(length: ...)` key `min` expects an integer literal",
            "unknown `@Validate(length: Length(...))` key `low`",
            "`@Validate(range: ...)` requires `min` to be <= `max`",
        ]
    );
}

#[test]
fn validates_nested_configuration() {
    let plugin = register_plugin();
    let mut address = class("Address");
    address.traits.clear();
    let mut profile = class("Profile");
    profile.fields = vec![
        field(
            "address",
            TypeIr::named("Address"),
            vec![validate("(nested: true)")],
        ),
        field(
            "tags",
            TypeIr::list_of(TypeIr::string()),
            vec![validate("(unknown: true)")],
        ),
    ];

    let diagnostics = plugin.validate(&library(vec![address, profile]));
    let messages = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.message.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        messages,
        vec![
            "`@Validate(nested: true)` target `Address` must derive `Validate()`",
            "unknown `@Validate` option `unknown`",
        ]
    );
}
