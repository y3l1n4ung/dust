use std::path::{Path, PathBuf};

use dust_dart_emit::render_template;
use dust_ir::DartFileIr;
use dust_plugin_api::AuxiliaryOutputContribution;
use serde::Serialize;

use crate::plugin::model::RouterSpec;

use super::{
    imports::{RouteImportKind, render_route_imports},
    metadata,
    navigation::{render_metadata_helpers, render_navigation_helpers, render_path_helpers},
    page_builder::{render_page_builder, render_shell_consistency_helpers},
    parser::render_parser,
    restore::render_restore_stack,
    route_classes::render_route_classes,
};

/// One generated route output file.
struct RouteGeneratedFile {
    /// Output path on disk.
    path: PathBuf,
    /// Complete generated source.
    source: String,
}

/// Template context for the generated route runtime file.
#[derive(Serialize)]
struct RouteFileContext<'a> {
    /// Generated router base class name.
    generated_base_class: &'a str,
    /// Generated sealed route path base class.
    route_path_class: &'a str,
    /// Generated route metadata list variable.
    routes_variable: &'a str,
    /// Generated route parser function.
    parse_route_function: &'a str,
    /// Generated route location helper function.
    route_location_function: &'a str,
    /// Generated route auth helper function.
    route_requires_auth_function: &'a str,
    /// Generated route branch helper function.
    route_branch_function: &'a str,
    /// Generated route debug helper function.
    route_debug_info_function: &'a str,
    /// Generated route guard helper function.
    route_guards_function: &'a str,
    /// Generated route page builder function.
    build_page_function: &'a str,
    /// Generated route stack restoration function.
    restore_stack_function: &'a str,
    /// Initial generated route class.
    initial_route_class: &'a str,
    /// Optional refresh listenable override.
    refresh_getter: String,
}

/// Renders all generated route files for a router spec.
pub(crate) fn render_route_generated_files(
    library: &DartFileIr,
    spec: &RouterSpec,
) -> Vec<AuxiliaryOutputContribution> {
    route_generated_files(library, spec)
        .into_iter()
        .map(|file| AuxiliaryOutputContribution {
            output_path: file.path,
            source: file.source,
        })
        .collect()
}

/// Renders all generated route files before converting to plugin contributions.
fn route_generated_files(library: &DartFileIr, spec: &RouterSpec) -> Vec<RouteGeneratedFile> {
    let output_dir = route_output_dir(library);
    let metadata_imports = render_route_imports(library, spec, RouteImportKind::Metadata);
    let runtime_imports = render_route_imports(library, spec, RouteImportKind::Runtime);
    let route_type_imports = render_route_imports(library, spec, RouteImportKind::RouteTypes);
    vec![
        RouteGeneratedFile {
            path: output_dir.join("routes.g.dart"),
            source: render_routes_file(),
        },
        RouteGeneratedFile {
            path: output_dir.join("paths.g.dart"),
            source: render_paths_file(spec, &route_type_imports),
        },
        RouteGeneratedFile {
            path: output_dir.join("metadata.g.dart"),
            source: render_metadata_file(spec, &metadata_imports),
        },
        RouteGeneratedFile {
            path: output_dir.join("navigation.g.dart"),
            source: render_navigation_file(spec, &route_type_imports),
        },
        RouteGeneratedFile {
            path: output_dir.join("runtime.g.dart"),
            source: render_runtime_file(spec, &runtime_imports),
        },
    ]
}

/// Renders the generated route barrel exported by the user-owned route entrypoint.
fn render_routes_file() -> String {
    generated_file(
        "",
        "",
        "export 'paths.g.dart';\nexport 'navigation.g.dart';\nexport 'metadata.g.dart';\nexport 'runtime.g.dart';\n",
    )
}

/// Returns the generated route output directory for one router source.
fn route_output_dir(library: &DartFileIr) -> PathBuf {
    let source_path = Path::new(&library.source_path);
    if source_path.is_absolute() {
        return source_path.with_extension("");
    }
    let package_root = Path::new(&library.package_root);
    if !package_root.as_os_str().is_empty() && package_root != Path::new(".") {
        return package_root.join(source_path).with_extension("");
    }

    let output_path = Path::new(&library.output_path);
    let Some(file_name) = output_path.file_name().and_then(|name| name.to_str()) else {
        return output_path.with_extension("");
    };
    if let Some(stem) = file_name.strip_suffix(".g.dart") {
        return output_path
            .parent()
            .map(|parent| parent.join(stem))
            .unwrap_or_else(|| PathBuf::from(stem));
    }
    output_path.with_extension("")
}

/// Renders generated route classes, path helpers, and the URI parser.
fn render_paths_file(spec: &RouterSpec, route_imports: &str) -> String {
    let mut body = String::new();
    render_route_classes(&mut body, spec);
    render_path_helpers(&mut body, spec);
    render_parser(&mut body, spec);
    generated_file(
        "import 'package:dust_flutter/route.dart';\n\n",
        route_imports,
        &body,
    )
}

/// Renders route metadata, debug helpers, guard lookup, and route table data.
fn render_metadata_file(spec: &RouterSpec, route_imports: &str) -> String {
    let mut body = String::new();
    metadata::render_route_metadata(&mut body, spec);
    render_metadata_helpers(&mut body, spec);
    generated_file(
        "import 'package:flutter/material.dart';\nimport 'package:flutter/cupertino.dart';\nimport 'package:dust_flutter/route.dart';\n\nimport 'paths.g.dart';\n",
        route_imports,
        &body,
    )
}

/// Renders BuildContext navigation helpers.
fn render_navigation_file(spec: &RouterSpec, route_imports: &str) -> String {
    let mut body = String::new();
    render_navigation_helpers(&mut body, spec);
    generated_file(
        "import 'package:flutter/widgets.dart';\nimport 'package:dust_flutter/route.dart';\n\nimport 'paths.g.dart';\n",
        route_imports,
        &body,
    )
}

/// Renders router base, stack restoration, shell checks, and page building.
fn render_runtime_file(spec: &RouterSpec, route_imports: &str) -> String {
    let mut body = String::new();
    body.push_str(&render_template(
        "route_runtime",
        include_str!("templates/route_runtime.jinja"),
        RouteFileContext {
            generated_base_class: &spec.generated_base_class,
            route_path_class: &spec.route_path_class,
            routes_variable: &spec.routes_variable,
            parse_route_function: &spec.parse_route_function,
            route_location_function: &spec.route_location_function,
            route_requires_auth_function: &spec.route_requires_auth_function,
            route_branch_function: &spec.route_branch_function,
            route_debug_info_function: &spec.route_debug_info_function,
            route_guards_function: &spec.route_guards_function,
            build_page_function: &spec.build_page_function,
            restore_stack_function: &spec.restore_stack_function,
            initial_route_class: &spec.initial_route_class,
            refresh_getter: render_refresh_getter(spec),
        },
    ));
    body.push_str("\n\n");
    render_restore_stack(&mut body, spec);
    render_shell_consistency_helpers(&mut body, spec);
    render_page_builder(&mut body, spec);
    generated_file(
        "import 'package:flutter/material.dart';\nimport 'package:flutter/cupertino.dart';\nimport 'package:dust_flutter/route.dart';\n\nimport 'paths.g.dart';\nimport 'metadata.g.dart';\n",
        route_imports,
        &body,
    )
}

/// Renders a generated Dart library with common route ignores.
fn generated_file(imports: &str, prelude: &str, body: &str) -> String {
    format!(
        "// GENERATED CODE - DO NOT MODIFY BY HAND.\n// Generated by dust.\n//\n// ignore_for_file: unused_import, unnecessary_import\n\n{imports}{prelude}{body}"
    )
}

/// Renders the router refresh-listenable override when available.
fn render_refresh_getter(spec: &RouterSpec) -> String {
    spec.refresh_listenable
        .as_ref()
        .map(|field| {
            format!(
                "  @override\n  Listenable? get refreshListenable => (this as dynamic).{} as Listenable?;",
                field
            )
        })
        .unwrap_or_default()
}
