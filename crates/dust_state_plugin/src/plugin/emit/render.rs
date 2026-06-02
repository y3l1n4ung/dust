use dust_ir::ClassIr;

use super::StateFieldSpec;

pub(super) fn render_view_model_output(
    class: &ClassIr,
    state_type: &str,
    args_type: &str,
    initial_source: Option<&str>,
    state_fields: &[StateFieldSpec],
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
    let base = render_base(&generated_base, state_type, args_type, &initial_state);
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
) -> String {
    format!(
        "abstract class {generated_base} extends ViewModelBase<{state_type}, {args_type}> {{\n  {generated_base}(super.args) : super(initialState: {initial_state});\n}}",
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
