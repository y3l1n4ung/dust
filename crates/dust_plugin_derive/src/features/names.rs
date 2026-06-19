use std::collections::BTreeSet;

use dust_ir::DartFileIr;

#[derive(Clone)]
pub(crate) struct NameAllocator {
    used: BTreeSet<String>,
}

impl NameAllocator {
    pub(crate) fn new<I, S>(reserved: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            used: reserved.into_iter().map(Into::into).collect(),
        }
    }

    pub(crate) fn allocate(&mut self, base: impl Into<String>) -> String {
        let base = base.into();
        if self.used.insert(base.clone()) {
            return base;
        }

        for suffix in 2.. {
            let candidate = format!("{base}{suffix}");
            if self.used.insert(candidate.clone()) {
                return candidate;
            }
        }

        unreachable!("unbounded suffix search must return")
    }
}

pub(crate) fn library_declaration_names(library: &DartFileIr) -> BTreeSet<String> {
    let mut names = BTreeSet::new();

    names.extend(library.classes.iter().map(|class| class.name.clone()));
    names.extend(library.mixins.iter().map(|mixin| mixin.name.source.clone()));
    names.extend(
        library
            .extensions
            .iter()
            .filter_map(|extension| extension.name.as_ref())
            .map(|name| name.source.clone()),
    );
    names.extend(
        library
            .extension_types
            .iter()
            .map(|extension_type| extension_type.name.source.clone()),
    );
    names.extend(
        library
            .functions
            .iter()
            .map(|function| function.name.source.clone()),
    );
    names.extend(
        library
            .variables
            .iter()
            .map(|variable| variable.name.source.clone()),
    );
    names.extend(
        library
            .typedefs
            .iter()
            .map(|typedef| typedef.name.source.clone()),
    );
    names.extend(library.enums.iter().map(|enum_ir| enum_ir.name.clone()));

    names
}

pub(crate) fn lower_first(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    format!(
        "{}{}",
        first.to_ascii_lowercase(),
        chars.collect::<String>()
    )
}

pub(crate) fn upper_first(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };

    format!(
        "{}{}",
        first.to_ascii_uppercase(),
        chars.collect::<String>()
    )
}
