use dust_dart_emit::render_template;
use serde::Serialize;

use super::shell::effective_shell;
use crate::plugin::model::RouteSpec;

/// Template context for the generated route metadata list.
#[derive(Serialize)]
struct MetadataListContext {
    /// Rendered metadata nodes in tree order.
    nodes: String,
}

/// Template context for one metadata list entry.
#[derive(Serialize)]
struct MetadataEntryContext {
    /// Indentation used by the generated Dart source.
    indent: String,
    /// Rendered metadata node source.
    node: String,
}

/// Template context for a generated metadata group node.
#[derive(Serialize)]
struct GeneratedGroupContext {
    /// Indentation used by the generated Dart source.
    indent: String,
    /// Path segment represented by this group.
    path: String,
    /// Rendered child metadata nodes.
    children: String,
}

/// Template context for a generated route metadata node.
#[derive(Serialize)]
struct GeneratedRouteContext {
    /// Indentation used by the generated Dart source.
    indent: String,
    /// Rendered generated route constructor fields.
    fields: String,
    /// Rendered child metadata nodes.
    children: String,
}

/// Template context for generated child metadata.
#[derive(Serialize)]
struct GeneratedChildrenContext {
    /// Optional prefix inserted before child lists.
    prefix: &'static str,
    /// Indentation used by the generated Dart source.
    indent: String,
    /// Rendered child nodes.
    nodes: String,
}

/// Renders the generated route metadata tree.
pub(super) fn render_route_metadata(out: &mut String, routes: &[RouteSpec]) {
    let tree = MetadataTree::build(routes);
    out.push_str(&render_template(
        "route_metadata_list",
        include_str!("templates/route_metadata_list.jinja"),
        MetadataListContext {
            nodes: render_metadata_nodes(&tree, routes, 1, true),
        },
    ));
    out.push_str("\n\n");
}

/// Prefix tree used to render nested route metadata.
#[derive(Debug, Default)]
struct MetadataTree {
    /// Index of the route represented by this node, if any.
    route_index: Option<usize>,
    /// Child path segments under this node.
    children: Vec<MetadataChild>,
}

/// Child node in the route metadata prefix tree.
#[derive(Debug)]
struct MetadataChild {
    /// Path segment represented by this child.
    segment: String,
    /// Subtree rooted at the segment.
    node: MetadataTree,
}

impl MetadataTree {
    /// Builds a metadata tree from sorted route specs.
    fn build(routes: &[RouteSpec]) -> Self {
        let mut root = Self::default();
        for (index, route) in routes.iter().enumerate() {
            let segments = route
                .path
                .split('/')
                .filter(|segment| !segment.is_empty())
                .collect::<Vec<_>>();
            if segments.is_empty() {
                root.route_index = Some(index);
            } else {
                root.insert(&segments, index);
            }
        }
        root
    }

    /// Inserts a route index under a path segment chain.
    fn insert(&mut self, segments: &[&str], route_index: usize) {
        let Some((segment, remaining)) = segments.split_first() else {
            self.route_index = Some(route_index);
            return;
        };
        let child_index = self
            .children
            .iter()
            .position(|child| child.segment == *segment)
            .unwrap_or_else(|| {
                self.children.push(MetadataChild {
                    segment: (*segment).to_owned(),
                    node: MetadataTree::default(),
                });
                self.children.len() - 1
            });
        self.children[child_index]
            .node
            .insert(remaining, route_index);
    }
}

/// Renders metadata nodes for a tree level.
fn render_metadata_nodes(
    node: &MetadataTree,
    routes: &[RouteSpec],
    indent: usize,
    root: bool,
) -> String {
    let mut entries = Vec::new();
    if let Some(index) = node.route_index {
        let children = if root { &[] } else { node.children.as_slice() };
        entries.push(render_metadata_entry(
            indent,
            render_generated_route(
                routes[index].path.as_str(),
                &routes[index],
                children,
                routes,
                indent,
            ),
        ));
    }
    for child in &node.children {
        let path = if root {
            format!("/{}", child.segment)
        } else {
            child.segment.clone()
        };
        let rendered = if let Some(index) = child.node.route_index {
            render_generated_route(&path, &routes[index], &child.node.children, routes, indent)
        } else {
            render_generated_group(&path, &child.node.children, routes, indent)
        };
        entries.push(render_metadata_entry(indent, rendered));
    }
    entries.join("\n")
}

/// Renders one metadata list entry.
fn render_metadata_entry(indent: usize, node: String) -> String {
    render_template(
        "route_metadata_entry",
        include_str!("templates/route_metadata_entry.jinja"),
        MetadataEntryContext {
            indent: indent_str(indent),
            node: node.trim_end().to_owned(),
        },
    )
}

/// Renders a generated metadata group for route children without a page.
fn render_generated_group(
    path: &str,
    children: &[MetadataChild],
    routes: &[RouteSpec],
    indent: usize,
) -> String {
    render_template(
        "generated_group",
        include_str!("templates/generated_group.jinja"),
        GeneratedGroupContext {
            indent: indent_str(indent),
            path: path.to_owned(),
            children: render_generated_children_with_prefix(children, routes, indent, true),
        },
    )
}

/// Renders a generated metadata node for a concrete route.
fn render_generated_route(
    path: &str,
    route: &RouteSpec,
    children: &[MetadataChild],
    routes: &[RouteSpec],
    indent: usize,
) -> String {
    let mut fields = vec![
        format!("{}  '{path}',\n", indent_str(indent)),
        format!("{}  page: {},\n", indent_str(indent), route.page_class),
        format!("{}  name: '{}',\n", indent_str(indent), route.name),
    ];
    if let Some(shell) = effective_shell(route, routes) {
        fields.push(format!("{}  shell: {shell},\n", indent_str(indent)));
    }
    if route.annotation.guards_configured {
        fields.push(format!(
            "{}  guards: [{}],\n",
            indent_str(indent),
            route.annotation.guards.join(", ")
        ));
    }
    if let Some(transition) = &route.annotation.transition {
        fields.push(format!(
            "{}  transition: {},\n",
            indent_str(indent),
            transition
                .strip_prefix("const ")
                .unwrap_or(transition.as_str())
        ));
    }
    if route.annotation.fullscreen_dialog {
        fields.push(format!("{}  fullscreenDialog: true,\n", indent_str(indent)));
    }
    if !route.annotation.maintain_state {
        fields.push(format!("{}  maintainState: false,\n", indent_str(indent)));
    }
    let children = render_generated_children_with_prefix(children, routes, indent, false);
    render_template(
        "generated_route",
        include_str!("templates/generated_route.jinja"),
        GeneratedRouteContext {
            indent: indent_str(indent),
            fields: fields.join(""),
            children,
        },
    )
}

/// Renders generated child metadata with an optional leading comma.
fn render_generated_children_with_prefix(
    children: &[MetadataChild],
    routes: &[RouteSpec],
    indent: usize,
    needs_prefix_comma: bool,
) -> String {
    if children.is_empty() {
        return String::new();
    }
    let nodes = children
        .iter()
        .map(|child| {
            let rendered = if let Some(index) = child.node.route_index {
                render_generated_route(
                    &child.segment,
                    &routes[index],
                    &child.node.children,
                    routes,
                    indent + 2,
                )
            } else {
                render_generated_group(&child.segment, &child.node.children, routes, indent + 2)
            };
            render_metadata_entry(indent + 2, rendered)
        })
        .collect::<Vec<_>>()
        .join("\n");
    render_template(
        "generated_children",
        include_str!("templates/generated_children.jinja"),
        GeneratedChildrenContext {
            prefix: if needs_prefix_comma { ",\n" } else { "" },
            indent: indent_str(indent),
            nodes,
        },
    )
}

/// Returns two-space indentation for generated Dart.
fn indent_str(indent: usize) -> String {
    "  ".repeat(indent)
}
