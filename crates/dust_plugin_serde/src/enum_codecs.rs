//! How generated code reaches an enum's JSON helpers from where it is used.
//!
//! An enum's `_$StatusSerialize` and `_$StatusDeserialize` helpers live in its
//! own library's part file, and Dart privacy is per library, so only code in
//! that library can call them. A class in another library has to go through
//! the public `$StatusSerializer` and `$StatusDeserializer` support types the
//! enum also gets. Before this split, anything outside the library fell through
//! to `.toJson()` and `Status.fromJson(...)`, which an enum does not have.

/// Enum names generated code can encode or decode, split by how it reaches them.
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct EnumCodecs<'a> {
    /// Enums declared in this library, reachable through their private helpers.
    pub(crate) local: &'a [&'a str],
    /// Enums declared anywhere in the workspace, from Pass 2 analysis.
    ///
    /// Includes this library's own enums; `local` is checked first, so those
    /// keep the shorter private form.
    pub(crate) workspace: &'a [String],
}

impl EnumCodecs<'_> {
    /// Returns the encode expression for `expr` when `name` is an enum this
    /// library can reach.
    pub(crate) fn encode(&self, name: &str, expr: &str) -> Option<String> {
        if self.local.contains(&name) {
            return Some(format!("_${name}Serialize({expr})"));
        }
        self.is_workspace_enum(name)
            .then(|| format!("const {}Serializer().serialize({expr})", support_type(name)))
    }

    /// Returns the decode expression for `raw` when `name` is an enum this
    /// library can reach.
    ///
    /// The public `Deserializer` interface takes the JSON value alone, so an
    /// enum from another library reports a bad value against the enum rather
    /// than against the field key. The value is still refused.
    pub(crate) fn decode(&self, name: &str, raw: &str, key: &str) -> Option<String> {
        if self.local.contains(&name) {
            return Some(format!("_${name}Deserialize({raw}, {key})"));
        }
        self.is_workspace_enum(name).then(|| {
            format!(
                "const {}Deserializer().deserialize({raw})",
                support_type(name)
            )
        })
    }

    /// Returns whether `name`, possibly import-prefixed, is a workspace enum.
    fn is_workspace_enum(&self, name: &str) -> bool {
        let (_, base) = split_prefix(name);
        self.workspace.iter().any(|known| known == base)
    }
}

/// Names the public support type for `name`, keeping any import prefix.
///
/// `s.Status` becomes `s.$Status`, since the prefix qualifies the library and
/// the `$` belongs to the type.
fn support_type(name: &str) -> String {
    match split_prefix(name) {
        (Some(prefix), base) => format!("{prefix}.${base}"),
        (None, base) => format!("${base}"),
    }
}

/// Splits an import prefix off a type name, if it has one.
fn split_prefix(name: &str) -> (Option<&str>, &str) {
    match name.rsplit_once('.') {
        Some((prefix, base)) => (Some(prefix), base),
        None => (None, name),
    }
}

#[cfg(test)]
mod tests {
    use super::EnumCodecs;

    fn codecs<'a>(local: &'a [&'a str], workspace: &'a [String]) -> EnumCodecs<'a> {
        EnumCodecs { local, workspace }
    }

    #[test]
    fn a_local_enum_keeps_its_private_helpers() {
        let workspace = vec!["Status".to_owned()];
        let codecs = codecs(&["Status"], &workspace);

        assert_eq!(
            codecs.encode("Status", "value").as_deref(),
            Some("_$StatusSerialize(value)")
        );
        assert_eq!(
            codecs.decode("Status", "raw", "'status'").as_deref(),
            Some("_$StatusDeserialize(raw, 'status')")
        );
    }

    #[test]
    fn an_enum_from_another_library_goes_through_its_public_support_types() {
        let workspace = vec!["Status".to_owned()];
        let codecs = codecs(&[], &workspace);

        assert_eq!(
            codecs.encode("Status", "value").as_deref(),
            Some("const $StatusSerializer().serialize(value)")
        );
        assert_eq!(
            codecs.decode("Status", "raw", "'status'").as_deref(),
            Some("const $StatusDeserializer().deserialize(raw)")
        );
    }

    #[test]
    fn an_import_prefix_qualifies_the_support_type() {
        let workspace = vec!["Status".to_owned()];
        let codecs = codecs(&[], &workspace);

        assert_eq!(
            codecs.encode("s.Status", "value").as_deref(),
            Some("const s.$StatusSerializer().serialize(value)")
        );
        assert_eq!(
            codecs.decode("s.Status", "raw", "'status'").as_deref(),
            Some("const s.$StatusDeserializer().deserialize(raw)")
        );
    }

    #[test]
    fn a_type_that_is_no_enum_is_left_to_the_caller() {
        let workspace = vec!["Status".to_owned()];
        let codecs = codecs(&["Status"], &workspace);

        assert_eq!(codecs.encode("Profile", "value"), None);
        assert_eq!(codecs.decode("Profile", "raw", "'profile'"), None);
    }
}
