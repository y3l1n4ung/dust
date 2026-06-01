use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use dust_dart_emit::DYNAMIC_TYPES;
use dust_ir::{ClassIr, LibraryIr, TypeIr};
use dust_plugin_api::{PluginContribution, SymbolPlan};

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
    if let Some(fields) = imported_class_fields(library, class_name) {
        return fields;
    }
    state_facts
        .get(class_name)
        .filter(|fields| !fields.is_empty())
        .cloned()
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
    let aspect_class = format!("_{}Aspect", class.name);
    let watch_name = format!("watch{}", class.name);
    let read_name = format!("read{}", class.name);
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
    let aspect = render_aspect_class(&aspect_class, &class.name, state_type, state_fields);
    let proxy = render_proxy(
        &proxy_class,
        &scope_class,
        state_type,
        &aspect_class,
        &class.name,
        state_fields,
    );
    let scope = render_scope(
        &scope_class,
        &inherited_class,
        &class.name,
        args_type,
        &aspect_class,
    );
    let inherited = render_inherited(&inherited_class, &class.name, state_type, &aspect_class);

    let listener = render_listener(
        &listener_class,
        &listener_state_class,
        &scope_class,
        &class.name,
    );
    let extension = format!(
        "extension {extension_class} on BuildContext {{\n  {proxy_class} {watch_name}() {{\n    return {proxy_class}(this);\n  }}\n\n  {vm} {read_name}() => {scope_class}.read(this);\n}}",
        vm = class.name,
    );

    [aspect, base, proxy, scope, inherited, listener, extension]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
}

fn render_aspect_class(
    aspect_class: &str,
    view_model_class: &str,
    state_type: &str,
    state_fields: &[StateFieldSpec],
) -> String {
    let field_selectors = state_fields
        .iter()
        .map(|field| {
            format!(
                "{ty} _{vm_lower}Select{field_pascal}({state_type} state) => state.{field_name};\nfinal _{vm_lower}{field_pascal}Aspect = {aspect_class}<{ty}>(\n  _{vm_lower}Select{field_pascal},\n);",
                ty = field.type_source,
                vm_lower = lower_camel(view_model_class),
                field_pascal = pascal_case(&field.name),
                field_name = field.name,
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    [format!(
        "final class {aspect_class}<R> {{\n  const {aspect_class}(this.selector);\n\n  final R Function({state_type} state) selector;\n\n  bool hasChanged({state_type} previous, {state_type} next) {{\n    return selector(previous) != selector(next);\n  }}\n}}",
    ), field_selectors]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join("\n\n")
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
        "abstract class {generated_base} extends ViewModelBase<{state_type}, {args_type}> {{\n  {generated_base}(super.args) : super(initialState: {initial_state});\n{getters}\n}}",
        getters = base_getters(state_fields, args_fields),
    )
}

fn render_proxy(
    proxy_class: &str,
    scope_class: &str,
    state_type: &str,
    aspect_class: &str,
    view_model_class: &str,
    state_fields: &[StateFieldSpec],
) -> String {
    let field_getters = state_fields
        .iter()
        .map(|field| {
            format!(
                "\n\n  {ty} get {name} {{\n    return {scope_class}.of(\n      _context,\n      aspect: _{vm_lower}{field_pascal}Aspect,\n    ).state.{name};\n  }}",
                ty = field.type_source,
                name = field.name,
                vm_lower = lower_camel(view_model_class),
                field_pascal = pascal_case(&field.name),
            )
        })
        .collect::<String>();
    format!(
        "class {proxy_class} {{\n  {proxy_class}(this._context);\n\n  final BuildContext _context;\n\n  {state_type} get value {{\n    return {scope_class}.of(_context).value;\n  }}{field_getters}\n\n  R select<R>(R Function({state_type} state) selector) {{\n    final aspect = {aspect_class}<R>(selector);\n    return selector({scope_class}.of(_context, aspect: aspect).value);\n  }}\n}}",
    )
}

fn render_scope(
    scope_class: &str,
    inherited_class: &str,
    view_model_class: &str,
    args_type: &str,
    aspect_class: &str,
) -> String {
    format!(
        "class {scope_class} extends StatefulWidget {{\n  const {scope_class}({{\n    super.key,\n    required this.args,\n    required this.create,\n    required this.child,\n  }}) : value = null;\n\n  const {scope_class}.value({{\n    super.key,\n    required {view_model_class} this.value,\n    required this.child,\n  }}) : args = null,\n       create = null;\n\n  final {args_type} Function(BuildContext context)? args;\n  final {view_model_class} Function(BuildContext context, {args_type} args)? create;\n  final {view_model_class}? value;\n  final Widget child;\n\n  static {view_model_class} read(BuildContext context) {{\n    final scope = context\n        .getElementForInheritedWidgetOfExactType<{inherited_class}>()\n        ?.widget as {inherited_class}?;\n    if (scope == null) throw StateError('No {scope_class} found in context.');\n    return scope.viewModel;\n  }}\n\n  static {view_model_class} of(BuildContext context, {{{aspect_class}<Object?>? aspect}}) {{\n    final scope = context.dependOnInheritedWidgetOfExactType<{inherited_class}>(\n      aspect: aspect,\n    );\n    if (scope == null) throw StateError('No {scope_class} found in context.');\n    return scope.viewModel;\n  }}\n\n  @override\n  State<{scope_class}> createState() => _{scope_class}State();\n}}\n\nclass _{scope_class}State extends State<{scope_class}> {{\n  {view_model_class}? _viewModel;\n  bool _ownsViewModel = false;\n\n  @override\n  void didChangeDependencies() {{\n    super.didChangeDependencies();\n    if (_viewModel == null) {{\n      _replaceViewModel(_resolveViewModel(), ownsViewModel: widget.value == null, notify: false);\n    }}\n  }}\n\n  @override\n  void didUpdateWidget({scope_class} oldWidget) {{\n    super.didUpdateWidget(oldWidget);\n    final external = widget.value;\n    if (external != null) {{\n      _replaceViewModel(external, ownsViewModel: false);\n    }} else if (oldWidget.value != null) {{\n      _replaceViewModel(_createOwnedViewModel(), ownsViewModel: true);\n    }}\n  }}\n\n  {view_model_class} _resolveViewModel() {{\n    return widget.value ?? _createOwnedViewModel();\n  }}\n\n  {view_model_class} _createOwnedViewModel() {{\n    final argsFactory = widget.args;\n    final create = widget.create;\n    if (argsFactory == null || create == null) {{\n      throw StateError('Owned {scope_class} requires args and create.');\n    }}\n    late final {view_model_class} created;\n    try {{\n      created = create(context, argsFactory(context));\n    }} catch (error, stackTrace) {{\n      Error.throwWithStackTrace(\n        StateError(\n          '{scope_class} failed to create its view model. Check the generated '\n          'scope args/create dependency injection. Original error: $error',\n        ),\n        stackTrace,\n      );\n    }}\n    return created;\n  }}\n\n  void _replaceViewModel(\n    {view_model_class} nextViewModel, {{\n    required bool ownsViewModel,\n    bool notify = true,\n  }}) {{\n    final previous = _viewModel;\n    if (identical(previous, nextViewModel)) {{\n      _ownsViewModel = ownsViewModel;\n      if (notify && mounted) setState(() {{}});\n      return;\n    }}\n    previous?.removeListener(_onViewModelStateChanged);\n    if (_ownsViewModel) previous?.dispose();\n    _viewModel = nextViewModel;\n    _ownsViewModel = ownsViewModel;\n    nextViewModel.addListener(_onViewModelStateChanged);\n    if (ownsViewModel) {{\n      scheduleMicrotask(() {{\n        if (mounted && identical(_viewModel, nextViewModel)) {{\n          nextViewModel.init();\n        }}\n      }});\n    }}\n    if (notify && mounted) setState(() {{}});\n  }}\n\n  void _onViewModelStateChanged() {{\n    if (mounted) setState(() {{}});\n  }}\n\n  @override\n  void dispose() {{\n    final viewModel = _viewModel;\n    viewModel?.removeListener(_onViewModelStateChanged);\n    if (_ownsViewModel) viewModel?.dispose();\n    super.dispose();\n  }}\n\n  @override\n  Widget build(BuildContext context) {{\n    final viewModel = _viewModel;\n    if (viewModel == null) {{\n      throw StateError('{scope_class} built before its view model was initialized.');\n    }}\n    return {inherited_class}(\n      viewModel: viewModel,\n      state: viewModel.value,\n      child: widget.child,\n    );\n  }}\n}}",
    )
}

fn render_inherited(
    inherited_class: &str,
    view_model_class: &str,
    state_type: &str,
    aspect_class: &str,
) -> String {
    format!(
        "class {inherited_class} extends InheritedModel<{aspect_class}<Object?>> {{\n  const {inherited_class}({{required this.viewModel, required this.state, required super.child}});\n\n  final {view_model_class} viewModel;\n  final {state_type} state;\n\n  /// Requires {state_type} to implement == and hashCode. Without value equality,\n  /// every emitted state is treated as changed and granular rebuilds degrade to\n  /// full dependent subtree rebuilds.\n  @override\n  bool updateShouldNotify({inherited_class} oldWidget) => state != oldWidget.state;\n\n  @override\n  bool updateShouldNotifyDependent(\n    {inherited_class} oldWidget,\n    Set<{aspect_class}<Object?>> dependencies,\n  ) {{\n    for (final aspect in dependencies) {{\n      if (aspect.hasChanged(oldWidget.state, state)) {{\n        return true;\n      }}\n    }}\n    return false;\n  }}\n}}",
    )
}

fn render_listener(
    listener_class: &str,
    listener_state_class: &str,
    scope_class: &str,
    view_model_class: &str,
) -> String {
    format!(
        "/// Listens to one-shot effects from {view_model_class}.\n///\n/// TODO: effects are Stream<Object> until ViewModelBase supports typed effect\n/// payloads through the @ViewModel annotation.\nclass {listener_class} extends StatefulWidget {{\n  const {listener_class}({{super.key, required this.listener, required this.child}});\n\n  final void Function(BuildContext context, Object effect) listener;\n  final Widget child;\n\n  @override\n  State<{listener_class}> createState() => {listener_state_class}();\n}}\n\nclass {listener_state_class} extends State<{listener_class}> {{\n  StreamSubscription<Object>? _sub;\n  {view_model_class}? _viewModel;\n\n  @override\n  void didChangeDependencies() {{\n    super.didChangeDependencies();\n    final nextViewModel = {scope_class}.read(context);\n    if (_viewModel == nextViewModel) return;\n    _sub?.cancel();\n    _viewModel = nextViewModel;\n    _sub = nextViewModel.effects.listen(_onEffect);\n  }}\n\n  void _onEffect(Object effect) {{\n    if (mounted) widget.listener(context, effect);\n  }}\n\n  @override\n  void dispose() {{\n    _sub?.cancel();\n    super.dispose();\n  }}\n\n  @override\n  Widget build(BuildContext context) => widget.child;\n}}",
    )
}

fn base_getters(state_fields: &[StateFieldSpec], args_fields: &[StateFieldSpec]) -> String {
    let state_getters = state_fields.iter().map(|field| {
        format!(
            "\n\n  {ty} get {name} => state.{name};",
            ty = field.type_source,
            name = field.name,
        )
    });
    let args_getters = args_fields
        .iter()
        .filter(|field| field.name != "observer")
        .map(|field| {
            format!(
                "\n\n  {ty} get {name} => args.{name};",
                ty = field.type_source,
                name = field.name,
            )
        });
    state_getters.chain(args_getters).collect::<String>()
}

fn render_type(ty: &TypeIr) -> String {
    DYNAMIC_TYPES.render(ty)
}

fn lower_camel(value: &str) -> String {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return String::new();
    };
    first.to_ascii_lowercase().to_string() + chars.as_str()
}

fn pascal_case(value: &str) -> String {
    value
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut chars = part.chars();
            let Some(first) = chars.next() else {
                return String::new();
            };
            first.to_ascii_uppercase().to_string() + chars.as_str()
        })
        .collect::<String>()
}
