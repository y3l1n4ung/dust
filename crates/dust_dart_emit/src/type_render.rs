use dust_ir::TypeIr;

/// Controls how unresolved Dust types render into Dart output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnknownTypeRendering {
    /// Render unresolved types as `dynamic`.
    Dynamic,
    /// Render unresolved types as `Object?` or `Object` for non-nullable output.
    ObjectNullable,
}

/// Shared Dart type renderer used across Dust emitters.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DartTypeRenderer {
    unknown: UnknownTypeRendering,
}

/// Renders unresolved types as `dynamic`.
pub const DYNAMIC_TYPES: DartTypeRenderer = DartTypeRenderer::new(UnknownTypeRendering::Dynamic);

/// Renders unresolved types as `Object?`.
pub const OBJECT_NULLABLE_TYPES: DartTypeRenderer =
    DartTypeRenderer::new(UnknownTypeRendering::ObjectNullable);

impl DartTypeRenderer {
    /// Creates a new renderer with the chosen unknown-type strategy.
    pub const fn new(unknown: UnknownTypeRendering) -> Self {
        Self { unknown }
    }

    /// Renders one normalized Dust type into Dart source.
    pub fn render(self, ty: &TypeIr) -> String {
        match ty {
            TypeIr::Builtin { kind, nullable } => {
                with_nullable(kind.as_str().to_owned(), *nullable)
            }
            TypeIr::Named {
                name,
                args,
                nullable,
            } => {
                let mut rendered = name.to_string();
                if !args.is_empty() {
                    rendered.push('<');
                    rendered.push_str(
                        &args
                            .iter()
                            .map(|arg| self.render(arg))
                            .collect::<Vec<_>>()
                            .join(", "),
                    );
                    rendered.push('>');
                }
                with_nullable(rendered, *nullable)
            }
            TypeIr::Function {
                signature,
                nullable,
            } => with_nullable(signature.to_string(), *nullable),
            TypeIr::Record { shape, nullable } => with_nullable(shape.to_string(), *nullable),
            TypeIr::Dynamic => "dynamic".to_owned(),
            TypeIr::Unknown => self.unknown.render(true).to_owned(),
        }
    }

    /// Renders one normalized Dust type into a non-nullable Dart source spelling.
    pub fn render_non_nullable(self, ty: &TypeIr) -> String {
        match ty {
            TypeIr::Builtin { kind, .. } => kind.as_str().to_owned(),
            TypeIr::Named { name, args, .. } => {
                if args.is_empty() {
                    name.to_string()
                } else {
                    format!(
                        "{}<{}>",
                        name,
                        args.iter()
                            .map(|arg| self.render(arg))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            TypeIr::Function { signature, .. } => signature.to_string(),
            TypeIr::Record { shape, .. } => shape.to_string(),
            TypeIr::Dynamic => "dynamic".to_owned(),
            TypeIr::Unknown => self.unknown.render(false).to_owned(),
        }
    }
}

impl UnknownTypeRendering {
    fn render(self, nullable: bool) -> &'static str {
        match self {
            Self::Dynamic => "dynamic",
            Self::ObjectNullable if nullable => "Object?",
            Self::ObjectNullable => "Object",
        }
    }
}

/// Returns the non-nullable shape of the provided normalized type.
pub fn non_nullable(ty: &TypeIr) -> TypeIr {
    match ty {
        TypeIr::Builtin { kind, .. } => TypeIr::builtin(*kind),
        TypeIr::Named { name, args, .. } => TypeIr::generic(name.as_ref(), args.to_vec()),
        TypeIr::Function { signature, .. } => TypeIr::function(signature.as_ref()),
        TypeIr::Record { shape, .. } => TypeIr::record(shape.as_ref()),
        TypeIr::Dynamic => TypeIr::dynamic(),
        TypeIr::Unknown => TypeIr::unknown(),
    }
}

fn with_nullable(mut rendered: String, nullable: bool) -> String {
    if nullable {
        rendered.push('?');
    }
    rendered
}

#[cfg(test)]
mod tests {
    use dust_ir::{BuiltinType, TypeIr};

    use super::{DYNAMIC_TYPES, OBJECT_NULLABLE_TYPES, non_nullable};

    #[test]
    fn renders_named_and_generic_types() {
        let ty = TypeIr::generic(
            "Map",
            vec![
                TypeIr::string(),
                TypeIr::list_of(TypeIr::named("Todo").nullable()),
            ],
        )
        .nullable();

        assert_eq!(
            OBJECT_NULLABLE_TYPES.render(&ty),
            "Map<String, List<Todo?>>?"
        );
        assert_eq!(
            OBJECT_NULLABLE_TYPES.render_non_nullable(&ty),
            "Map<String, List<Todo?>>"
        );
    }

    #[test]
    fn renders_unknown_types_per_strategy() {
        assert_eq!(DYNAMIC_TYPES.render(&TypeIr::unknown()), "dynamic");
        assert_eq!(OBJECT_NULLABLE_TYPES.render(&TypeIr::unknown()), "Object?");
        assert_eq!(
            OBJECT_NULLABLE_TYPES.render_non_nullable(&TypeIr::unknown()),
            "Object"
        );
    }

    #[test]
    fn drops_nullability_from_type_shapes() {
        let ty = TypeIr::Builtin {
            kind: BuiltinType::String,
            nullable: true,
        };

        assert_eq!(non_nullable(&ty), TypeIr::string());
    }
}
