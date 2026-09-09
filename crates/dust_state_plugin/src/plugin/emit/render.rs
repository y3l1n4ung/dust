use dust_dart_emit::render_template;
use dust_ir::ClassIr;
use serde::Serialize;

use crate::plugin::model::ViewModelMode;

/// Template contexts for the generated state widgets.
mod contexts;
pub(crate) use self::contexts::*;

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

/// Renders the async generated base class for a view model.
fn render_async_base(
    generated_base: &str,
    view_model_class: &str,
    data_type: &str,
    args_type: &str,
) -> String {
    render_template(
        "base_async_class",
        include_str!("templates/base_async_class.jinja"),
        AsyncBaseContext {
            generated_base,
            view_model_class,
            data_type,
            args_type,
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

/// Renders the selector widget.
fn render_selector(
    selector_class: &str,
    selector_state_class: &str,
    scope_class: &str,
    view_model_class: &str,
    state_type: &str,
) -> String {
    render_template(
        "selector_class",
        include_str!("templates/selector_class.jinja"),
        SelectorContext {
            selector_class,
            selector_state_class,
            scope_class,
            view_model_class,
            state_type,
        },
    )
}

/// Renders the async builder widget.
fn render_async_builder(builder_class: &str, view_model_class: &str, data_type: &str) -> String {
    let previous_data_type = previous_data_type(data_type);
    let previous_data_argument = previous_data_argument(data_type);
    render_template(
        "async_builder_class",
        include_str!("templates/async_builder_class.jinja"),
        AsyncBuilderContext {
            builder_class,
            view_model_class,
            data_type,
            previous_data_type: &previous_data_type,
            previous_data_argument: &previous_data_argument,
        },
    )
}

/// Returns the nullable type used by async error callbacks.
fn previous_data_type(data_type: &str) -> String {
    if data_type.trim_end().ends_with('?') {
        data_type.to_owned()
    } else {
        format!("{data_type}?")
    }
}

/// Returns the previous-data expression passed to async data builders.
fn previous_data_argument(data_type: &str) -> String {
    if data_type.trim_end().ends_with('?') {
        "previousData".to_owned()
    } else {
        format!("previousData as {data_type}")
    }
}

/// Renders the scope widget that owns a view model instance.
fn render_scope(
    scope_class: &str,
    instance_class: &str,
    inherited_class: &str,
    view_model_class: &str,
    args_type: &str,
) -> String {
    render_template(
        "scope_class",
        include_str!("templates/scope_class.jinja"),
        ScopeContext {
            scope_class,
            instance_class,
            inherited_class,
            view_model_class,
            args_type,
        },
    )
}

/// Renders the identity-only inherited widget.
fn render_instance(instance_class: &str, view_model_class: &str) -> String {
    render_template(
        "instance_class",
        include_str!("templates/instance_class.jinja"),
        InstanceContext {
            instance_class,
            view_model_class,
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

#[cfg(test)]
mod tests {
    use super::render_async_builder;

    #[test]
    fn async_builder_uses_nullable_previous_data_type_once() {
        let source =
            render_async_builder("ProfileViewModelBuilder", "ProfileViewModel", "Profile?");

        assert!(source.contains("Profile? previousData"));
        assert!(source.contains("data(context, previousData)"));
        assert!(!source.contains("Profile??"));
        assert!(!source.contains("previousData as Profile?"));
    }
}
