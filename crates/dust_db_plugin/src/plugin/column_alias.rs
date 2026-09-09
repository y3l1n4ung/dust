//! SQLx's column-alias overrides, and the names they leave behind.
//!
//! Describe reports what the database inferred, and inference is sometimes
//! wrong about nullability in ways no schema can express: a `LEFT JOIN` makes a
//! `NOT NULL` column nullable in the result, and `max()` over an empty set is
//! `NULL` while `count(*)` is not. SQLx answers this with markers in the column
//! alias, which works because SQL allows arbitrary text there.
//!
//! ```sql
//! SELECT o.total as "total?",   -- nullable, whatever the schema says
//!        max(o.id) as "id!"     -- not null, whatever inference says
//! ```
//!
//! The alias is the name the database reports, so the marker has to come off
//! before the column is matched against a row class — otherwise `total?` looks
//! like a column no field asked for.

/// What a column alias asks for beyond its name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum NullabilityOverride {
    /// `foo!` — treat as non-null however the database described it.
    NotNull,
    /// `foo?` — treat as nullable however the database described it.
    Nullable,
    /// No marker; the described nullability stands.
    Inferred,
}

/// A described column name split into the name and what it overrides.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ColumnAlias {
    /// Column name with any marker removed, as a row class spells it.
    pub(crate) name: String,
    /// What the alias asks for.
    pub(crate) nullability: NullabilityOverride,
}

/// Splits a described column name into its name and any override marker.
///
/// A bare `foo` is left alone, so a query that uses no markers behaves as it
/// did. Only a trailing `!` or `?` is a marker: a column genuinely named with
/// one is not expressible in a row class field anyway.
pub(crate) fn parse_column_alias(described: &str) -> ColumnAlias {
    let (name, nullability) = match described.as_bytes().last() {
        Some(b'!') => (
            &described[..described.len() - 1],
            NullabilityOverride::NotNull,
        ),
        Some(b'?') => (
            &described[..described.len() - 1],
            NullabilityOverride::Nullable,
        ),
        _ => (described, NullabilityOverride::Inferred),
    };
    ColumnAlias {
        name: name.to_owned(),
        nullability,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_bare_name_overrides_nothing() {
        assert_eq!(
            parse_column_alias("total"),
            ColumnAlias {
                name: "total".to_owned(),
                nullability: NullabilityOverride::Inferred,
            }
        );
    }

    #[test]
    fn a_marker_is_read_and_removed() {
        assert_eq!(
            parse_column_alias("total!"),
            ColumnAlias {
                name: "total".to_owned(),
                nullability: NullabilityOverride::NotNull,
            }
        );
        assert_eq!(
            parse_column_alias("total?"),
            ColumnAlias {
                name: "total".to_owned(),
                nullability: NullabilityOverride::Nullable,
            }
        );
    }

    #[test]
    fn a_marker_in_the_middle_is_part_of_the_name() {
        // Only a trailing marker overrides, so a name that merely contains one
        // is left alone rather than silently truncated.
        assert_eq!(parse_column_alias("we!rd").name, "we!rd");
        assert_eq!(
            parse_column_alias("we!rd").nullability,
            NullabilityOverride::Inferred
        );
    }

    #[test]
    fn an_empty_name_is_not_treated_as_a_marker() {
        assert_eq!(parse_column_alias("").name, "");
        assert_eq!(parse_column_alias("!").name, "");
    }
}
