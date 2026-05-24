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
    let state_facts = state_facts(plan);
    for class in &library.classes {
        let Some(config) = view_model_config(&class.configs) else {
            continue;
        };
        let Some(annotation) = parse_view_model_annotation(config.arguments_source.as_deref())
        else {
            continue;
        };
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

fn state_facts(plan: &SymbolPlan) -> Vec<StateFact> {
    plan.workspace_string_set(STATES_ANALYSIS_KEY)
        .unwrap_or_default()
        .iter()
        .filter_map(|value| serde_json::from_str::<StateFact>(value).ok())
        .collect()
}

fn class_fields(
    library: &LibraryIr,
    state_facts: &[StateFact],
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
        .iter()
        .find(|fact| fact.class_name == class_name)
        .map(|fact| {
            fact.fields
                .iter()
                .map(|field| StateFieldSpec {
                    name: field.name.clone(),
                    type_source: field.type_source.clone(),
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
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
                "      if (aspect == {aspect_class}.{name} &&\n          state.{name} != oldWidget.state.{name}) {{\n        return true;\n      }}\n",
                name = field.name,
            ));
        }
        format!("    for (final aspect in dependencies) {{\n{checks}    }}\n    return false;\n")
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

fn base_getters(state_fields: &[StateFieldSpec], args_fields: &[StateFieldSpec]) -> String {
    let state_getters = state_fields
        .iter()
        .map(|field| {
            format!(
                "  {ty} get {name} => state.{name};\n",
                ty = field.type_source,
                name = field.name
            )
        })
        .collect::<String>();
    let args_getters = args_fields
        .iter()
        .filter(|field| field.name != "observer")
        .map(|field| {
            format!(
                "  {ty} get {name} => args.{name};\n",
                ty = field.type_source,
                name = field.name
            )
        })
        .collect::<String>();
    format!("{state_getters}{args_getters}")
}

fn render_type(ty: &TypeIr) -> String {
    DYNAMIC_TYPES.render(ty)
}
