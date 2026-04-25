/// Built-in Dart types represented without heap-allocated names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinType {
    /// The built-in `String` type.
    String,
    /// The built-in `int` type.
    Int,
    /// The built-in `bool` type.
    Bool,
    /// The built-in `double` type.
    Double,
    /// The built-in `num` type.
    Num,
    /// The built-in `Object` type.
    Object,
}

impl BuiltinType {
    /// Returns the Dart source spelling for this built-in type.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::String => "String",
            Self::Int => "int",
            Self::Bool => "bool",
            Self::Double => "double",
            Self::Num => "num",
            Self::Object => "Object",
        }
    }
}

/// A normalized type shape relevant to Dust generation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeIr {
    /// One of Dart's common built-in types.
    Builtin {
        /// The built-in type kind.
        kind: BuiltinType,
        /// Whether the type is nullable.
        nullable: bool,
    },
    /// A named Dart type, optionally generic and nullable.
    Named {
        /// The base type name.
        name: Box<str>,
        /// Generic type arguments.
        args: Box<[TypeIr]>,
        /// Whether the type is nullable.
        nullable: bool,
    },
    /// A function type kept distinct from named types.
    Function {
        /// The exact Dart function type source without a trailing `?`.
        signature: Box<str>,
        /// Whether the type is nullable.
        nullable: bool,
    },
    /// A record type kept distinct from named types.
    Record {
        /// The exact Dart record type source without a trailing `?`.
        shape: Box<str>,
        /// Whether the type is nullable.
        nullable: bool,
    },
    /// The explicit Dart `dynamic` type.
    Dynamic,
    /// A fallback type used when the source shape cannot be normalized yet.
    Unknown,
}

impl TypeIr {
    /// Creates the explicit Dart `dynamic` type.
    pub const fn dynamic() -> Self {
        Self::Dynamic
    }

    /// Creates the fallback unresolved type.
    pub const fn unknown() -> Self {
        Self::Unknown
    }

    /// Creates a non-nullable built-in type.
    pub const fn builtin(kind: BuiltinType) -> Self {
        Self::Builtin {
            kind,
            nullable: false,
        }
    }

    /// Creates a non-nullable named type with no generic arguments.
    pub fn named(name: impl Into<Box<str>>) -> Self {
        Self::Named {
            name: name.into(),
            args: Box::new([]),
            nullable: false,
        }
    }

    /// Creates the built-in `String` type.
    pub const fn string() -> Self {
        Self::builtin(BuiltinType::String)
    }

    /// Creates the built-in `int` type.
    pub const fn int() -> Self {
        Self::builtin(BuiltinType::Int)
    }

    /// Creates the built-in `bool` type.
    pub const fn bool() -> Self {
        Self::builtin(BuiltinType::Bool)
    }

    /// Creates the built-in `double` type.
    pub const fn double() -> Self {
        Self::builtin(BuiltinType::Double)
    }

    /// Creates the built-in `num` type.
    pub const fn num() -> Self {
        Self::builtin(BuiltinType::Num)
    }

    /// Creates the built-in `Object` type.
    pub const fn object() -> Self {
        Self::builtin(BuiltinType::Object)
    }

    /// Creates a non-nullable generic named type.
    pub fn generic(name: impl Into<Box<str>>, args: Vec<TypeIr>) -> Self {
        Self::Named {
            name: name.into(),
            args: args.into_boxed_slice(),
            nullable: false,
        }
    }

    /// Creates a non-nullable function type from its exact Dart signature.
    pub fn function(signature: impl Into<Box<str>>) -> Self {
        Self::Function {
            signature: signature.into(),
            nullable: false,
        }
    }

    /// Creates a non-nullable record type from its exact Dart source.
    pub fn record(shape: impl Into<Box<str>>) -> Self {
        Self::Record {
            shape: shape.into(),
            nullable: false,
        }
    }

    /// Creates a `List<T>` type.
    pub fn list_of(item: TypeIr) -> Self {
        Self::generic("List", vec![item])
    }

    /// Creates a `Map<K, V>` type.
    pub fn map_of(key: TypeIr, value: TypeIr) -> Self {
        Self::generic("Map", vec![key, value])
    }

    /// Returns a nullable version of this type.
    pub fn nullable(self) -> Self {
        match self {
            Self::Builtin { kind, .. } => Self::Builtin {
                kind,
                nullable: true,
            },
            Self::Named { name, args, .. } => Self::Named {
                name,
                args,
                nullable: true,
            },
            Self::Function { signature, .. } => Self::Function {
                signature,
                nullable: true,
            },
            Self::Record { shape, .. } => Self::Record {
                shape,
                nullable: true,
            },
            Self::Dynamic => Self::Dynamic,
            Self::Unknown => Self::Unknown,
        }
    }

    /// Returns the base type name for named and built-in types.
    pub fn name(&self) -> Option<&str> {
        match self {
            Self::Builtin { kind, .. } => Some(kind.as_str()),
            Self::Named { name, .. } => Some(name),
            Self::Function { .. } | Self::Record { .. } | Self::Dynamic | Self::Unknown => None,
        }
    }

    /// Returns the generic arguments for named types.
    pub fn args(&self) -> &[TypeIr] {
        match self {
            Self::Named { args, .. } => args,
            Self::Builtin { .. }
            | Self::Function { .. }
            | Self::Record { .. }
            | Self::Dynamic
            | Self::Unknown => &[],
        }
    }

    /// Returns `true` when this is a named or built-in type with the expected base name.
    pub fn is_named(&self, expected: &str) -> bool {
        self.name() == Some(expected)
    }

    /// Returns `true` when this is a specific built-in type.
    pub fn is_builtin(&self, expected: BuiltinType) -> bool {
        matches!(self, Self::Builtin { kind, .. } if *kind == expected)
    }

    /// Returns `true` if this type is nullable.
    pub const fn is_nullable(&self) -> bool {
        match self {
            Self::Builtin { nullable, .. }
            | Self::Named { nullable, .. }
            | Self::Function { nullable, .. }
            | Self::Record { nullable, .. } => *nullable,
            Self::Dynamic | Self::Unknown => false,
        }
    }

    /// Returns `true` if this is a function type.
    pub const fn is_function(&self) -> bool {
        matches!(self, Self::Function { .. })
    }

    /// Returns `true` if this is a record type.
    pub const fn is_record(&self) -> bool {
        matches!(self, Self::Record { .. })
    }
}
