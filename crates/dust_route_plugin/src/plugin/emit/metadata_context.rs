use serde::Serialize;

/// Template context for the generated route metadata list.
#[derive(Serialize)]
pub(super) struct MetadataListContext {
    /// Generated route metadata list variable.
    pub(super) routes_variable: String,
    /// Rendered metadata nodes in tree order.
    pub(super) nodes: String,
}

/// Template context for one metadata list entry.
#[derive(Serialize)]
pub(super) struct MetadataEntryContext {
    /// Indentation used by the generated Dart source.
    pub(super) indent: String,
    /// Rendered metadata node source.
    pub(super) node: String,
}

/// Template context for a generated metadata group node.
#[derive(Serialize)]
pub(super) struct GeneratedGroupContext {
    /// Indentation used by the generated Dart source.
    pub(super) indent: String,
    /// Path segment represented by this group.
    pub(super) path: String,
    /// Rendered child metadata nodes.
    pub(super) children: String,
}

/// Template context for a generated route metadata node.
#[derive(Serialize)]
pub(super) struct GeneratedRouteContext {
    /// Indentation used by the generated Dart source.
    pub(super) indent: String,
    /// Rendered generated route constructor fields.
    pub(super) fields: String,
    /// Rendered child metadata nodes.
    pub(super) children: String,
}

/// Template context for generated child metadata.
#[derive(Serialize)]
pub(super) struct GeneratedChildrenContext {
    /// Optional prefix inserted before child lists.
    pub(super) prefix: &'static str,
    /// Indentation used by the generated Dart source.
    pub(super) indent: String,
    /// Rendered child nodes.
    pub(super) nodes: String,
}
