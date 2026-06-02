use std::path::Path;

use dust_ir::{LibraryIr, TypeIr};

pub(super) fn write_indent(out: &mut String, indent: usize) {
    out.push_str(&indent_str(indent));
}

fn indent_str(indent: usize) -> String {
    "  ".repeat(indent)
}

pub(super) enum RenderedField {
    Line(String),
    Inline(String),
}

impl RenderedField {
    pub(super) fn line(value: impl Into<String>) -> Self {
        Self::Line(value.into())
    }

    pub(super) fn inline(value: impl Into<String>) -> Self {
        Self::Inline(value.into())
    }

    pub(super) fn render(self, out: &mut String, indent: usize) {
        match self {
            Self::Line(value) => {
                write_indent(out, indent);
                out.push_str(&value);
                out.push('\n');
            }
            Self::Inline(value) => {
                if !out.ends_with('\n') {
                    out.push('\n');
                }
                write_indent(out, indent);
                out.push_str(&value);
            }
        }
    }
}
pub(super) fn package_import_uri(library: &LibraryIr) -> Option<String> {
    let source = Path::new(&library.source_path);
    let relative = source
        .strip_prefix(&library.package_root)
        .ok()
        .and_then(|path| path.strip_prefix("lib").ok())
        .or_else(|| source.strip_prefix("lib").ok())?;
    let path = relative
        .components()
        .filter_map(|component| match component {
            std::path::Component::Normal(value) => value.to_str(),
            _ => None,
        })
        .collect::<Vec<_>>()
        .join("/");
    Some(format!("package:{}/{}", library.package_name, path))
}

pub(super) fn dart_type(ty: &TypeIr) -> String {
    match ty {
        TypeIr::Builtin { kind, nullable } => {
            format!("{}{}", kind.as_str(), if *nullable { "?" } else { "" })
        }
        TypeIr::Named { name, nullable, .. } => {
            format!("{name}{}", if *nullable { "?" } else { "" })
        }
        TypeIr::Dynamic => "dynamic".to_owned(),
        TypeIr::Function {
            signature,
            nullable,
        } => format!("{signature}{}", if *nullable { "?" } else { "" }),
        TypeIr::Record { shape, nullable } => {
            format!("{shape}{}", if *nullable { "?" } else { "" })
        }
        TypeIr::Unknown => "Object?".to_owned(),
    }
}

pub(super) fn upper_camel_identifier(value: &str) -> String {
    value
        .rsplit('.')
        .next()
        .unwrap_or(value)
        .split(|ch: char| ch == '_' || ch == '-' || ch.is_whitespace())
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            match chars.next() {
                Some(first) => first.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect()
}
