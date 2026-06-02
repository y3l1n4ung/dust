use super::{
    formatting::{RenderedField, write_indent},
    shell::effective_shell,
};
use crate::plugin::model::RouteSpec;

pub(super) fn render_route_metadata(out: &mut String, routes: &[RouteSpec]) {
    out.push_str("const List<GeneratedRoute> $appRoutes = [\n");
    let tree = MetadataTree::build(routes);
    render_metadata_nodes(out, &tree, routes, 1, true);
    out.push_str("];\n\n");
}

#[derive(Debug, Default)]
struct MetadataTree {
    route_index: Option<usize>,
    children: Vec<MetadataChild>,
}

#[derive(Debug)]
struct MetadataChild {
    segment: String,
    node: MetadataTree,
}

impl MetadataTree {
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

fn render_metadata_nodes(
    out: &mut String,
    node: &MetadataTree,
    routes: &[RouteSpec],
    indent: usize,
    root: bool,
) {
    if let Some(index) = node.route_index {
        write_indent(out, indent);
        let children = if root { &[] } else { node.children.as_slice() };
        render_generated_route(
            out,
            routes[index].path.as_str(),
            &routes[index],
            children,
            routes,
            indent,
        );
        out.push_str(",\n");
    }
    for child in &node.children {
        write_indent(out, indent);
        let path = if root {
            format!("/{}", child.segment)
        } else {
            child.segment.clone()
        };
        if let Some(index) = child.node.route_index {
            render_generated_route(
                out,
                &path,
                &routes[index],
                &child.node.children,
                routes,
                indent,
            );
        } else {
            render_generated_group(out, &path, &child.node.children, routes, indent);
        }
        out.push_str(",\n");
    }
}

fn render_generated_group(
    out: &mut String,
    path: &str,
    children: &[MetadataChild],
    routes: &[RouteSpec],
    indent: usize,
) {
    out.push_str("GeneratedRoute(\n");
    write_indent(out, indent + 1);
    out.push_str(&format!("'{path}'"));
    render_generated_children(out, children, routes, indent);
    out.push('\n');
    write_indent(out, indent);
    out.push(')');
}

fn render_generated_route(
    out: &mut String,
    path: &str,
    route: &RouteSpec,
    children: &[MetadataChild],
    routes: &[RouteSpec],
    indent: usize,
) {
    out.push_str("GeneratedRoute(\n");
    let mut fields = vec![
        RenderedField::line(format!("'{path}',")),
        RenderedField::line(format!("page: {},", route.page_class)),
        RenderedField::inline(format!("name: '{}',", route.name)),
    ];
    if let Some(shell) = effective_shell(route, routes) {
        fields.push(RenderedField::inline(format!("shell: {shell},")));
    }
    if route.annotation.guards_configured {
        fields.push(RenderedField::inline(format!(
            "guards: [{}],",
            route.annotation.guards.join(", ")
        )));
    }
    if let Some(transition) = &route.annotation.transition {
        fields.push(RenderedField::inline(format!(
            "transition: {},",
            transition
                .strip_prefix("const ")
                .unwrap_or(transition.as_str())
        )));
    }
    if route.annotation.fullscreen_dialog {
        fields.push(RenderedField::inline("fullscreenDialog: true,"));
    }
    if !route.annotation.maintain_state {
        fields.push(RenderedField::inline("maintainState: false,"));
    }
    for field in fields {
        field.render(out, indent + 1);
    }
    render_generated_children(out, children, routes, indent);
    out.push('\n');
    write_indent(out, indent);
    out.push(')');
}

fn render_generated_children(
    out: &mut String,
    children: &[MetadataChild],
    routes: &[RouteSpec],
    indent: usize,
) {
    if children.is_empty() {
        return;
    }
    if !out.ends_with(',') {
        out.push(',');
    }
    out.push('\n');
    write_indent(out, indent + 1);
    out.push_str("routes: [\n");
    for child in children {
        write_indent(out, indent + 2);
        if let Some(index) = child.node.route_index {
            render_generated_route(
                out,
                &child.segment,
                &routes[index],
                &child.node.children,
                routes,
                indent + 2,
            );
        } else {
            render_generated_group(
                out,
                &child.segment,
                &child.node.children,
                routes,
                indent + 2,
            );
        }
        out.push_str(",\n");
    }
    write_indent(out, indent + 1);
    out.push_str("],");
}
