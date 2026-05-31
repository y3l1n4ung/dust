use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use dust_dart_emit::DYNAMIC_TYPES;
use dust_ir::{ClassIr, LibraryIr, TypeIr};
use dust_plugin_api::{PluginContribution, SymbolPlan};
use heck::ToLowerCamelCase;

use super::{
    constants::STATES_ANALYSIS_KEY,
    model::StateFact,
    parse::{parse_view_model_annotation, view_model_config},
};

#[derive(Debug, Clone, PartialEq, Eq)]
struct StateFieldSpec {
    name: String,
    type_source: String,
}

pub(crate) fn emit_library_state(library: &LibraryIr, plan: &SymbolPlan) -> PluginContribution {
    let mut contribution = PluginContribution::default();
    let view_models = library
        .classes
        .iter()
        .filter_map(|class| {
            let config = view_model_config(&class.configs)?;
            let annotation = parse_view_model_annotation(config.arguments_source.as_deref())?;
            Some((class, annotation))
        })
        .collect::<Vec<_>>();
    if view_models.is_empty() {
        return contribution;
    }

    let state_facts = state_facts(plan);
    for (class, annotation) in view_models {
        let args_type = annotation
            .args_type
            .clone()
            .unwrap_or_else(|| "ViewModelArgs".to_owned());
        let state_fields = class_fields(library, &state_facts, &annotation.state_type);
        let args_fields = class_fields(library, &state_facts, &args_type);

        contribution.support_types.push(render_view_model_output(
            class,
            &annotation.state_type,
            &args_type,
            annotation.initial_source.as_deref(),
            &state_fields,
            &args_fields,
        ));
    }
    contribution
}

fn state_facts(plan: &SymbolPlan) -> HashMap<String, Vec<StateFieldSpec>> {
    plan.workspace_string_set(STATES_ANALYSIS_KEY)
        .unwrap_or_default()
        .iter()
        .filter_map(|value| serde_json::from_str::<StateFact>(value).ok())
        .map(|fact| {
            let fields = fact
                .fields
                .into_iter()
                .map(|field| StateFieldSpec {
                    name: field.name,
                    type_source: field.type_source,
                })
                .collect::<Vec<_>>();
            (fact.class_name, fields)
        })
        .collect()
}

fn class_fields(
    library: &LibraryIr,
    state_facts: &HashMap<String, Vec<StateFieldSpec>>,
    class_name: &str,
) -> Vec<StateFieldSpec> {
    if let Some(class) = library
        .classes
        .iter()
        .find(|candidate| candidate.name == class_name)
    {
        let fields = class
            .fields
            .iter()
            .map(|field| StateFieldSpec {
                name: field.name.clone(),
                type_source: render_type(&field.ty),
            })
            .collect::<Vec<_>>();
        if !fields.is_empty() {
            return fields;
        }
    }
    state_facts
        .get(class_name)
        .filter(|fields| !fields.is_empty())
        .cloned()
        .or_else(|| imported_class_fields(library, class_name))
        .unwrap_or_default()
}

fn imported_class_fields(library: &LibraryIr, class_name: &str) -> Option<Vec<StateFieldSpec>> {
    library
        .imports
        .iter()
        .filter_map(|uri| resolve_import_path(library, uri))
        .find_map(|path| {
            let source = fs::read_to_string(path).ok()?;
            fields_from_source_class(&source, class_name)
        })
        .filter(|fields| !fields.is_empty())
}

fn resolve_import_path(library: &LibraryIr, uri: &str) -> Option<PathBuf> {
    if uri.starts_with("dart:") || uri.starts_with("package:flutter/") {
        return None;
    }
    if let Some(rest) = uri.strip_prefix("package:") {
        let (package, path) = rest.split_once('/')?;
        if package == library.package_name {
            return Some(Path::new(&library.package_root).join("lib").join(path));
        }
        return None;
    }
    let source_dir = Path::new(&library.package_root)
        .join(&library.source_path)
        .parent()?
        .to_path_buf();
    Some(normalize_path(&source_dir.join(uri)))
}

fn fields_from_source_class(source: &str, class_name: &str) -> Option<Vec<StateFieldSpec>> {
    let class_offset = source.find(&format!("class {class_name}"))?;
    let body_start = source[class_offset..].find('{')? + class_offset;
    let body_end = matching_brace(source, body_start)?;
    let body = &source[body_start + 1..body_end];
    let declared_names = declared_type_names(source);
    let fields = body
        .lines()
        .filter_map(|line| field_from_line(line, &declared_names))
        .collect::<Vec<_>>();
    Some(fields)
}

fn field_from_line(line: &str, declared_names: &[String]) -> Option<StateFieldSpec> {
    let mut line = line.trim();
    if line.starts_with('@') || line.starts_with("//") || line.contains('(') || !line.ends_with(';')
    {
        return None;
    }
    line = line.trim_end_matches(';').trim();
    let declaration = line.split('=').next().unwrap_or(line).trim();
    let parts = declaration.split_whitespace().collect::<Vec<_>>();
    if parts.len() < 2 {
        return None;
    }
    if !parts
        .iter()
        .any(|part| matches!(*part, "final" | "var" | "late"))
    {
        return None;
    }
    let name = parts.last()?.trim_end_matches(';');
    if matches!(name, "get" | "set") {
        return None;
    }
    let type_parts = parts[..parts.len() - 1]
        .iter()
        .copied()
        .filter(|part| !matches!(*part, "static" | "late" | "final" | "var" | "const"))
        .collect::<Vec<_>>();
    if type_parts.is_empty() {
        return None;
    }
    Some(StateFieldSpec {
        name: name.to_owned(),
        type_source: sanitize_imported_type(&type_parts.join(" "), declared_names),
    })
}

fn declared_type_names(source: &str) -> Vec<String> {
    source
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            let rest = line
                .strip_prefix("class ")
                .or_else(|| line.strip_prefix("final class "))
                .or_else(|| line.strip_prefix("sealed class "))
                .or_else(|| line.strip_prefix("enum "))?;
            Some(
                rest.split(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
                    .next()
                    .unwrap_or_default()
                    .to_owned(),
            )
        })
        .filter(|name| !name.is_empty())
        .collect()
}

fn sanitize_imported_type(type_source: &str, declared_names: &[String]) -> String {
    let ty = type_source.trim();
    if let Some(inner) = ty
        .strip_prefix("List<")
        .and_then(|value| value.strip_suffix('>'))
    {
        return if is_visible_imported_type(inner.trim(), declared_names) {
            ty.to_owned()
        } else {
            "List<Object?>".to_owned()
        };
    }
    if ty.contains('<') {
        return "Object?".to_owned();
    }
    if is_visible_imported_type(ty.trim_end_matches('?'), declared_names) {
        ty.to_owned()
    } else {
        "Object?".to_owned()
    }
}

fn is_visible_imported_type(type_name: &str, declared_names: &[String]) -> bool {
    matches!(
        type_name,
        "String" | "int" | "double" | "num" | "bool" | "DateTime" | "Object" | "dynamic" | "void"
    ) || declared_names.iter().any(|name| name == type_name)
}

fn matching_brace(source: &str, open: usize) -> Option<usize> {
    let mut depth = 0_i32;
    for (offset, ch) in source[open..].char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(open + offset);
                }
            }
            _ => {}
        }
    }
    None
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            std::path::Component::ParentDir => {
                out.pop();
            }
            std::path::Component::CurDir => {}
            other => out.push(other.as_os_str()),
        }
    }
    out
}

fn render_view_model_output(
    class: &ClassIr,
    state_type: &str,
    args_type: &str,
    initial_source: Option<&str>,
    state_fields: &[StateFieldSpec],
    args_fields: &[StateFieldSpec],
) -> String {
    let generated_base = format!("${}", class.name);
    let proxy_class = format!("_${}Proxy", class.name);
    let scope_class = format!("{}Scope", class.name);
    let inherited_class = format!("_{}Inherited", class.name);
    let listener_class = format!("{}Listener", class.name);
    let listener_state_class = format!("_{}ListenerState", class.name);
    let extension_class = format!("{}BuildContext", class.name);
    let watch_name = format!("watch{}", class.name);
    let read_name = format!("read{}", class.name);
    let context_getter_name = class.name.to_lower_camel_case();
    let aspect_class = format!("_{}Aspect", class.name);
    let aspect_enum = render_aspect_enum(&aspect_class, state_fields);
    let initial_state = initial_source
        .map(str::to_owned)
        .unwrap_or_else(|| format!("const {state_type}()"));
    let base = render_base(
        &generated_base,
        state_type,
        args_type,
        &initial_state,
        state_fields,
        args_fields,
    );
    let proxy = render_proxy(
        &proxy_class,
        &scope_class,
        &class.name,
        state_type,
        &aspect_class,
        state_fields,
    );
    let scope = render_scope(&scope_class, &inherited_class, &class.name, args_type);
    let inherited = render_inherited(
        &inherited_class,
        &class.name,
        state_type,
        &aspect_class,
        state_fields,
    );
    let listener = render_listener(&listener_class, &listener_state_class, &scope_class);
    let extension = format!(
        "extension {extension_class} on BuildContext {{\n  {vm} get {context_getter_name} => {scope_class}.of(this);\n\n  {proxy_class} {watch_name}() {{\n    return {proxy_class}(this, {scope_class}.read(this));\n  }}\n\n  {vm} {read_name}() => {scope_class}.read(this);\n}}",
        vm = class.name,
    );

    [
        aspect_enum,
        base,
        proxy,
        scope,
        inherited,
        listener,
        extension,
    ]
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join("\n\n")
}

fn render_aspect_enum(aspect_class: &str, state_fields: &[StateFieldSpec]) -> String {
    if state_fields.is_empty() {
        return String::new();
    }
    let variants = state_fields
        .iter()
        .map(|field| field.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    format!("enum {aspect_class} {{ {variants} }}")
}

fn render_base(
    generated_base: &str,
    state_type: &str,
    args_type: &str,
    initial_state: &str,
    state_fields: &[StateFieldSpec],
    args_fields: &[StateFieldSpec],
) -> String {
    format!(
        "abstract class {generated_base} extends ViewModelBase<{state_type}, {args_type}> {{\n  {generated_base}(super.args) : super(initialState: {initial_state});\n{getters}}}",
        getters = base_getters(state_fields, args_fields),
    )
}

fn render_proxy(
    proxy_class: &str,
    scope_class: &str,
    view_model_class: &str,
    state_type: &str,
    aspect_class: &str,
    state_fields: &[StateFieldSpec],
) -> String {
    let mut source = format!(
        "class {proxy_class} {{\n  {proxy_class}(this._context, this._vm);\n\n  final BuildContext _context;\n  final {view_model_class} _vm;\n\n  {state_type} get value {{\n    {scope_class}.of(_context);\n    return _vm.value;\n  }}\n",
    );
    for field in state_fields {
        source.push_str(&format!(
            "\n  {ty} get {name} {{\n    {scope_class}.of(_context, aspect: {aspect_class}.{name});\n    return _vm.state.{name};\n  }}\n",
            ty = field.type_source,
            name = field.name,
        ));
    }
    source.push('}');
    source
}

fn render_scope(
    scope_class: &str,
    inherited_class: &str,
    view_model_class: &str,
    args_type: &str,
) -> String {
    format!(
        "class {scope_class} extends StatefulWidget {{\n  const {scope_class}({{\n    super.key,\n    required this.args,\n    required this.create,\n    required this.child,\n  }}) : value = null;\n\n  const {scope_class}.value({{\n    super.key,\n    required {view_model_class} this.value,\n    required this.child,\n  }}) : args = null,\n       create = null;\n\n  final {args_type} Function(BuildContext context)? args;\n  final {view_model_class} Function(BuildContext context, {args_type} args)? create;\n  final {view_model_class}? value;\n  final Widget child;\n\n  static {view_model_class} read(BuildContext context) {{\n    final scope = context\n        .getElementForInheritedWidgetOfExactType<{inherited_class}>()\n        ?.widget as {inherited_class}?;\n    if (scope == null) throw StateError('No {scope_class} found in context.');\n    return scope.viewModel;\n  }}\n\n  static {view_model_class} of(BuildContext context, {{Object? aspect}}) {{\n    final scope = context.dependOnInheritedWidgetOfExactType<{inherited_class}>(\n      aspect: aspect,\n    );\n    if (scope == null) throw StateError('No {scope_class} found in context.');\n    return scope.viewModel;\n  }}\n\n  @override\n  State<{scope_class}> createState() => _{scope_class}State();\n}}\n\nclass _{scope_class}State extends State<{scope_class}> {{\n  @override\n  Widget build(BuildContext context) {{\n    final external = widget.value;\n    return external == null\n        ? ViewModelOwner<{view_model_class}, {args_type}>(\n            debugName: '{scope_class}',\n            args: widget.args!,\n            create: widget.create!,\n            builder: _buildInherited,\n          )\n        : ViewModelOwner<{view_model_class}, {args_type}>.value(\n            debugName: '{scope_class}.value',\n            value: external,\n            builder: _buildInherited,\n          );\n  }}\n\n  Widget _buildInherited(BuildContext context, {view_model_class} viewModel) {{\n    return ListenableBuilder(\n      listenable: viewModel,\n      builder: (context, child) => {inherited_class}(\n        viewModel: viewModel,\n        state: viewModel.value,\n        child: child!,\n      ),\n      child: widget.child,\n    );\n  }}\n}}",
    )
}

fn render_inherited(
    inherited_class: &str,
    view_model_class: &str,
    state_type: &str,
    aspect_class: &str,
    state_fields: &[StateFieldSpec],
) -> String {
    let dependent_body = if state_fields.is_empty() {
        "    return false;\n".to_owned()
    } else {
        let mut checks = String::new();
        for field in state_fields {
            checks.push_str(&format!(
                "        case {aspect_class}.{name}:\n          if (state.{name} != oldWidget.state.{name}) {{\n            return true;\n          }}\n          break;\n",
                name = field.name,
            ));
        }
        format!(
            "    for (final aspect in dependencies) {{\n      switch (aspect) {{\n{checks}        default:\n          break;\n      }}\n    }}\n    return false;\n"
        )
    };
    format!(
        "class {inherited_class} extends InheritedModel<Object> {{\n  const {inherited_class}({{required this.viewModel, required this.state, required super.child}});\n\n  final {view_model_class} viewModel;\n  final {state_type} state;\n\n  @override\n  bool updateShouldNotify({inherited_class} oldWidget) => state != oldWidget.state;\n\n  @override\n  bool updateShouldNotifyDependent({inherited_class} oldWidget, Set<Object> dependencies) {{\n{dependent_body}  }}\n}}",
    )
}

fn render_listener(listener_class: &str, listener_state_class: &str, scope_class: &str) -> String {
    format!(
        "class {listener_class} extends StatefulWidget {{\n  const {listener_class}({{super.key, required this.listener, required this.child}});\n\n  final void Function(BuildContext context, Object effect) listener;\n  final Widget child;\n\n  @override\n  State<{listener_class}> createState() => {listener_state_class}();\n}}\n\nclass {listener_state_class} extends State<{listener_class}> {{\n  StreamSubscription<Object>? _sub;\n\n  @override\n  void didChangeDependencies() {{\n    super.didChangeDependencies();\n    _sub?.cancel();\n    _sub = {scope_class}.read(context).effects.listen((effect) {{\n      if (mounted) widget.listener(context, effect);\n    }});\n  }}\n\n  @override\n  void dispose() {{\n    _sub?.cancel();\n    super.dispose();\n  }}\n\n  @override\n  Widget build(BuildContext context) => widget.child;\n}}",
    )
}

fn base_getters(_state_fields: &[StateFieldSpec], _args_fields: &[StateFieldSpec]) -> String {
    String::new()
}

fn render_type(ty: &TypeIr) -> String {
    DYNAMIC_TYPES.render(ty)
}
