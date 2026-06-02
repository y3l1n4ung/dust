use crate::plugin::model::{RouteParamSpec, RouteSpec, RouterSpec};

use super::formatting::{dart_type, upper_camel_identifier};

pub(super) fn render_helpers(out: &mut String, spec: &RouterSpec) {
    out.push_str("String routeLocation(AppRoutePath route) => route.location;\n\n");
    out.push_str("String _routePath(\n");
    out.push_str("  List<String> segments, {\n");
    out.push_str("  Map<String, String>? queryParameters,\n");
    out.push_str("}) {\n");
    out.push_str("  final query = queryParameters?.isEmpty ?? true ? null : queryParameters;\n");
    out.push_str(
        "  final text = Uri(pathSegments: segments, queryParameters: query).toString();\n",
    );
    out.push_str("  if (text.isEmpty) return '/';\n");
    out.push_str("  return text.startsWith('/') ? text : '/$text';\n");
    out.push_str("}\n\n");
    out.push_str("bool routeRequiresAuth(AppRoutePath route) => route.requiresAuth;\n\n");
    out.push_str("List<RouteGuard<AppRoutePath>> routeGuards(\n");
    out.push_str("  AppRoutePath route,\n");
    out.push_str("  $AppRouter router,\n");
    out.push_str(") {\n");
    out.push_str("  return switch (route) {\n");
    for route in &spec.routes {
        if route.annotation.guards.is_empty() {
            continue;
        }
        let pattern = format!("{}()", route.route_class);
        let guards = route
            .annotation
            .guards
            .iter()
            .map(|guard| format!("router.create{}()", upper_camel_identifier(guard)))
            .collect::<Vec<_>>()
            .join(", ");
        out.push_str(&format!("    {pattern} => [{guards}],\n"));
    }
    out.push_str("    _ => const [],\n");
    out.push_str("  };\n");
    out.push_str("}\n\n");
    out.push_str("extension DustRouterContext on BuildContext {\n");
    out.push_str("  DustRouterController<AppRoutePath> get router =>\n");
    out.push_str("      DustRouter.of<AppRoutePath>(this);\n");
    out.push_str("  AppRoutesNavigation get routes => AppRoutesNavigation(router);\n");
    out.push_str("}\n\n");
    out.push_str("final class AppRoutesNavigation {\n");
    out.push_str("  const AppRoutesNavigation(this._router);\n\n");
    out.push_str("  final DustRouterController<AppRoutePath> _router;\n\n");
    for route in &spec.routes {
        let route_ctor = format!("{}({})", route.route_class, render_route_args(route));
        let params = render_factory_params(route);
        let factory = format!("RouteNavigation<AppRoutePath> {}({params})", route.name,);
        let body = format!("RouteNavigation(_router, {route_ctor})");
        if factory.len() + body.len() + 7 <= 80 {
            out.push_str(&format!("  {factory} => {body};\n\n"));
        } else {
            out.push_str(&format!("  {factory} =>\n      {body};\n\n"));
        }
    }
    out.push_str("  void pop() => _router.pop();\n");
    out.push_str("}\n\n");
    out.push_str("final class RouteNavigation<T extends Object> {\n");
    out.push_str("  const RouteNavigation(this._router, this.route);\n\n");
    out.push_str("  final DustRouterController<T> _router;\n  final T route;\n\n");
    out.push_str("  void go() => _router.go(route);\n  void push() => _router.push(route);\n  void replace() => _router.replace(route);\n");
    out.push_str("}\n\n");
}
fn render_factory_params(route: &RouteSpec) -> String {
    let params = route
        .params
        .iter()
        .map(render_factory_param)
        .collect::<Vec<_>>()
        .join(", ");
    if route.params.iter().any(|param| param.is_named) {
        format!("{{{params}}}")
    } else {
        params
    }
}

fn render_factory_param(param: &RouteParamSpec) -> String {
    let ty = dart_type(&param.ty);
    if param.is_path || (!param.ty.is_nullable() && !param.has_default) {
        format!("required {ty} {}", param.name)
    } else if let Some(default_value) = &param.default_value_source {
        format!("{ty} {} = {default_value}", param.name)
    } else {
        format!("{ty} {}", param.name)
    }
}

fn render_route_args(route: &RouteSpec) -> String {
    route
        .params
        .iter()
        .map(|param| format!("{}: {}", param.name, param.name))
        .collect::<Vec<_>>()
        .join(", ")
}
