//! Template contexts for the generated state widgets.

use super::*;

/// Template context for the generated abstract view model base class.
#[derive(Serialize)]
pub(super) struct BaseContext<'a> {
    /// Generated base class name that user view models extend.
    pub(super) generated_base: &'a str,
    /// User-authored view model class name.
    pub(super) view_model_class: &'a str,
    /// Dart state type managed by the view model.
    pub(super) state_type: &'a str,
    /// Dart args type passed to the generated base.
    pub(super) args_type: &'a str,
    /// Dart expression used to initialize state.
    pub(super) initial_state: &'a str,
}

/// Template context for the generated async view model base class.
#[derive(Serialize)]
pub(super) struct AsyncBaseContext<'a> {
    /// Generated base class name that user view models extend.
    pub(super) generated_base: &'a str,
    /// User-authored view model class name.
    pub(super) view_model_class: &'a str,
    /// Dart data type loaded by the view model.
    pub(super) data_type: &'a str,
    /// Dart args type passed to the generated base.
    pub(super) args_type: &'a str,
}

/// Template context for the generated build context proxy class.
#[derive(Serialize)]
pub(super) struct ProxyContext<'a> {
    /// Generated proxy class name.
    pub(super) proxy_class: &'a str,
    /// Generated scope class name used for lookups.
    pub(super) scope_class: &'a str,
    /// User-authored view model class name.
    pub(super) view_model_class: &'a str,
    /// Dart state type managed by the view model.
    pub(super) state_type: &'a str,
}

/// Template context for the generated selector widget.
#[derive(Serialize)]
pub(super) struct SelectorContext<'a> {
    /// Generated public selector widget class name.
    pub(super) selector_class: &'a str,
    /// Generated private selector state class name.
    pub(super) selector_state_class: &'a str,
    /// Generated scope class name used for lookups.
    pub(super) scope_class: &'a str,
    /// User-authored view model class name.
    pub(super) view_model_class: &'a str,
    /// Dart state type managed by the view model.
    pub(super) state_type: &'a str,
}

/// Template context for the generated async builder widget.
#[derive(Serialize)]
pub(super) struct AsyncBuilderContext<'a> {
    /// Generated public builder widget class name.
    pub(super) builder_class: &'a str,
    /// User-authored view model class name.
    pub(super) view_model_class: &'a str,
    /// Dart data type loaded by the view model.
    pub(super) data_type: &'a str,
    /// Dart type used for optional previous data.
    pub(super) previous_data_type: &'a str,
    /// Dart expression used to pass previous data to the data builder.
    pub(super) previous_data_argument: &'a str,
}

/// Template context for the generated view model scope widget.
#[derive(Serialize)]
pub(super) struct ScopeContext<'a> {
    /// Generated public scope class name.
    pub(super) scope_class: &'a str,
    /// Generated private instance inherited widget class name.
    pub(super) instance_class: &'a str,
    /// Generated private inherited widget class name.
    pub(super) inherited_class: &'a str,
    /// User-authored view model class name.
    pub(super) view_model_class: &'a str,
    /// Dart args type accepted by the scope.
    pub(super) args_type: &'a str,
}

/// Template context for the generated identity-only inherited widget.
#[derive(Serialize)]
pub(super) struct InstanceContext<'a> {
    /// Generated inherited widget class name.
    pub(super) instance_class: &'a str,
    /// User-authored view model class name.
    pub(super) view_model_class: &'a str,
}

/// Template context for the generated inherited widget.
#[derive(Serialize)]
pub(super) struct InheritedContext<'a> {
    /// Generated inherited widget class name.
    pub(super) inherited_class: &'a str,
    /// User-authored view model class name.
    pub(super) view_model_class: &'a str,
    /// Dart state type exposed through the inherited widget.
    pub(super) state_type: &'a str,
}

/// Template context for the generated listener widget.
#[derive(Serialize)]
pub(super) struct ListenerContext<'a> {
    /// Generated listener widget class name.
    pub(super) listener_class: &'a str,
    /// Generated private listener state class name.
    pub(super) listener_state_class: &'a str,
    /// Generated scope class name used for subscription lookup.
    pub(super) scope_class: &'a str,
    /// User-authored view model class name.
    pub(super) view_model_class: &'a str,
}

/// Template context for the generated build context extension.
#[derive(Serialize)]
pub(super) struct ExtensionContext<'a> {
    /// Generated extension name.
    pub(super) extension_class: &'a str,
    /// Generated proxy class returned by watch helpers.
    pub(super) proxy_class: &'a str,
    /// Generated watch helper method name.
    pub(super) watch_name: &'a str,
    /// User-authored view model class name.
    pub(super) view_model_class: &'a str,
    /// Generated read helper method name.
    pub(super) read_name: &'a str,
    /// Generated scope class used for lookup.
    pub(super) scope_class: &'a str,
}

/// Renders the full generated support block for one view model class.
pub(crate) fn render_view_model_output(
    class: &ClassIr,
    state_type: &str,
    args_type: &str,
    initial_source: Option<&str>,
    mode: ViewModelMode,
) -> String {
    let generated_base = format!("${}", class.name);
    let proxy_class = format!("_${}Proxy", class.name);
    let selector_class = format!("{}Selector", class.name);
    let selector_state_class = format!("_{}SelectorState", class.name);
    let builder_class = format!("{}Builder", class.name);
    let scope_class = format!("{}Scope", class.name);
    let instance_class = format!("_{}Instance", class.name);
    let inherited_class = format!("_{}Inherited", class.name);
    let listener_class = format!("{}Listener", class.name);
    let listener_state_class = format!("_{}ListenerState", class.name);
    let extension_class = format!("{}BuildContext", class.name);
    let watch_name = format!("watch{}", class.name);
    let read_name = format!("read{}", class.name);
    let initial_state = initial_source
        .map(str::to_owned)
        .unwrap_or_else(|| format!("const {state_type}()"));
    let generated_state_type = match mode {
        ViewModelMode::Sync => state_type.to_owned(),
        ViewModelMode::Async => format!("AsyncState<{state_type}>"),
    };
    let base = match mode {
        ViewModelMode::Sync => render_base(
            &generated_base,
            &class.name,
            state_type,
            args_type,
            &initial_state,
        ),
        ViewModelMode::Async => {
            render_async_base(&generated_base, &class.name, state_type, args_type)
        }
    };
    let proxy = render_proxy(
        &proxy_class,
        &scope_class,
        &class.name,
        &generated_state_type,
    );
    let selector = render_selector(
        &selector_class,
        &selector_state_class,
        &scope_class,
        &class.name,
        &generated_state_type,
    );
    let async_builder = match mode {
        ViewModelMode::Sync => None,
        ViewModelMode::Async => Some(render_async_builder(
            &builder_class,
            &class.name,
            state_type,
        )),
    };
    let scope = render_scope(
        &scope_class,
        &instance_class,
        &inherited_class,
        &class.name,
        args_type,
    );
    let instance = render_instance(&instance_class, &class.name);
    let inherited = render_inherited(&inherited_class, &class.name, &generated_state_type);
    let listener = render_listener(
        &listener_class,
        &listener_state_class,
        &scope_class,
        &class.name,
    );
    let extension = render_template(
        "context_extension",
        include_str!("../templates/context_extension.jinja"),
        ExtensionContext {
            extension_class: &extension_class,
            proxy_class: &proxy_class,
            watch_name: &watch_name,
            view_model_class: &class.name,
            read_name: &read_name,
            scope_class: &scope_class,
        },
    );

    let mut sections = vec![base, proxy, selector];
    if let Some(async_builder) = async_builder {
        sections.push(async_builder);
    }
    sections.extend([scope, instance, inherited, listener, extension]);
    sections
        .into_iter()
        .map(|section| section.trim().to_owned())
        .collect::<Vec<_>>()
        .join("\n\n")
}
