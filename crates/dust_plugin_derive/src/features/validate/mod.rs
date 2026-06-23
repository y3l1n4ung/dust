/// Renders validation mixin and support source.
mod emit;
/// Parses and stores validation annotation config.
mod model;
/// Renders rule snippets used by validation templates.
mod rule_snippets;
/// Validates validation annotation shapes.
mod rules;
/// Renders Dart type source and input kind metadata.
mod type_source;

pub(crate) use emit::emit_validate;
pub(crate) use rules::validate_validate;
