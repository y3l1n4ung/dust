use dust_dart_emit::render_template;
use dust_ir::ClassIr;
use serde::Serialize;

/// Root template context for a generated view model support block.
#[derive(Serialize)]
struct ViewModelOutputContext {
    /// Rendered generated base class.
    base: String,
    /// Rendered proxy object exposed from build context helpers.
    proxy: String,
    /// Rendered scope widget.
    scope: String,
    /// Rendered inherited widget backing the scope.
    inherited: String,
    /// Rendered listener widget.
    listener: String,
    /// Rendered build context extension.
    extension: String,
}

/// Template context for the generated abstract view model base class.
#[derive(Serialize)]
struct BaseContext<'a> {
    /// Generated base class name that user view models extend.
    generated_base: &'a str,
    /// User-authored view model class name.
    view_model_class: &'a str,
    /// Dart state type managed by the view model.
    state_type: &'a str,
    /// Dart args type passed to the generated base.
    args_type: &'a str,
    /// Dart expression used to initialize state.
    initial_state: &'a str,
}

/// Template context for the generated build context proxy class.
#[derive(Serialize)]
struct ProxyContext<'a> {
    /// Generated proxy class name.
    proxy_class: &'a str,
    /// Generated scope class name used for lookups.
    scope_class: &'a str,
    /// User-authored view model class name.
    view_model_class: &'a str,
    /// Dart state type managed by the view model.
    state_type: &'a str,
}

/// Template context for the generated view model scope widget.
#[derive(Serialize)]
struct ScopeContext<'a> {
    /// Generated public scope class name.
    scope_class: &'a str,
    /// Generated private inherited widget class name.
    inherited_class: &'a str,
    /// User-authored view model class name.
    view_model_class: &'a str,
    /// Dart args type accepted by the scope.
    args_type: &'a str,
}

/// Template context for the generated inherited widget.
#[derive(Serialize)]
struct InheritedContext<'a> {
    /// Generated inherited widget class name.
    inherited_class: &'a str,
    /// User-authored view model class name.
    view_model_class: &'a str,
    /// Dart state type exposed through the inherited widget.
    state_type: &'a str,
}

/// Template context for the generated listener widget.
#[derive(Serialize)]
struct ListenerContext<'a> {
    /// Generated listener widget class name.
    listener_class: &'a str,
    /// Generated private listener state class name.
    listener_state_class: &'a str,
    /// Generated scope class name used for subscription lookup.
    scope_class: &'a str,
    /// User-authored view model class name.
    view_model_class: &'a str,
}

/// Template context for the generated build context extension.
#[derive(Serialize)]
struct ExtensionContext<'a> {
    /// Generated extension name.
    extension_class: &'a str,
    /// Generated proxy class returned by watch helpers.
    proxy_class: &'a str,
    /// Generated watch helper method name.
    watch_name: &'a str,
    /// User-authored view model class name.
    view_model_class: &'a str,
    /// Generated read helper method name.
    read_name: &'a str,
    /// Generated scope class used for lookup.
    scope_class: &'a str,
}

/// Renders the full generated support block for one view model class.
pub(super) fn render_view_model_output(
    class: &ClassIr,
    state_type: &str,
    args_type: &str,
    initial_source: Option<&str>,
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
    let initial_state = initial_source
        .map(str::to_owned)
        .unwrap_or_else(|| format!("const {state_type}()"));
    let base = render_base(
        &generated_base,
        &class.name,
        state_type,
        args_type,
        &initial_state,
    );
    let proxy = render_proxy(&proxy_class, &scope_class, &class.name, state_type);
    let scope = render_scope(&scope_class, &inherited_class, &class.name, args_type);
    let inherited = render_inherited(&inherited_class, &class.name, state_type);
    let listener = render_listener(
        &listener_class,
        &listener_state_class,
        &scope_class,
        &class.name,
    );
    let extension = render_template(
        "context_extension",
        include_str!("templates/context_extension.jinja"),
        ExtensionContext {
            extension_class: &extension_class,
            proxy_class: &proxy_class,
            watch_name: &watch_name,
            view_model_class: &class.name,
            read_name: &read_name,
            scope_class: &scope_class,
        },
    );

    render_template(
        "view_model_output",
        include_str!("templates/view_model_output.jinja"),
        ViewModelOutputContext {
            base,
            proxy,
            scope,
            inherited,
            listener,
            extension,
        },
    )
}

/// Renders the abstract generated base class for a view model.
fn render_base(
    generated_base: &str,
    view_model_class: &str,
    state_type: &str,
    args_type: &str,
    initial_state: &str,
) -> String {
    render_template(
        "base_class",
        include_str!("templates/base_class.jinja"),
        BaseContext {
            generated_base,
            view_model_class,
            state_type,
            args_type,
            initial_state,
        },
    )
}

/// Renders the build context proxy.
fn render_proxy(
    proxy_class: &str,
    scope_class: &str,
    view_model_class: &str,
    state_type: &str,
) -> String {
    render_template(
        "proxy_class",
        include_str!("templates/proxy_class.jinja"),
        ProxyContext {
            proxy_class,
            scope_class,
            view_model_class,
            state_type,
        },
    )
}

/// Renders the scope widget that owns a view model instance.
fn render_scope(
    scope_class: &str,
    inherited_class: &str,
    view_model_class: &str,
    args_type: &str,
) -> String {
    render_template(
        "scope_class",
        include_str!("templates/scope_class.jinja"),
        ScopeContext {
            scope_class,
            inherited_class,
            view_model_class,
            args_type,
        },
    )
}

/// Renders the inherited widget used for full-state rebuilds.
fn render_inherited(inherited_class: &str, view_model_class: &str, state_type: &str) -> String {
    render_template(
        "inherited_class",
        include_str!("templates/inherited_class.jinja"),
        InheritedContext {
            inherited_class,
            view_model_class,
            state_type,
        },
    )
}

/// Renders a listener widget for view model state changes.
fn render_listener(
    listener_class: &str,
    listener_state_class: &str,
    scope_class: &str,
    view_model_class: &str,
) -> String {
    render_template(
        "listener_class",
        include_str!("templates/listener_class.jinja"),
        ListenerContext {
            listener_class,
            listener_state_class,
            scope_class,
            view_model_class,
        },
    )
}
