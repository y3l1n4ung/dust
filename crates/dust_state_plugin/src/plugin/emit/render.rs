use dust_dart_emit::render_template;
use dust_ir::ClassIr;
use serde::Serialize;

use super::StateFieldSpec;

#[derive(Serialize)]
struct ViewModelOutputContext {
    aspect: String,
    base: String,
    proxy: String,
    scope: String,
    inherited: String,
    listener: String,
    extension: String,
}

#[derive(Serialize)]
struct AspectContext<'a> {
    aspect_class: &'a str,
    state_type: &'a str,
}

#[derive(Serialize)]
struct AspectSelectorContext<'a> {
    aspect_class: &'a str,
    state_type: &'a str,
    vm_lower: String,
    field_pascal: String,
    field_name: &'a str,
    field_type: &'a str,
}

#[derive(Serialize)]
struct BaseContext<'a> {
    generated_base: &'a str,
    view_model_class: &'a str,
    state_type: &'a str,
    args_type: &'a str,
    initial_state: &'a str,
}

#[derive(Serialize)]
struct ProxyContext<'a> {
    proxy_class: &'a str,
    scope_class: &'a str,
    view_model_class: &'a str,
    state_type: &'a str,
    aspect_class: &'a str,
    field_getters: String,
}

#[derive(Serialize)]
struct ProxyFieldContext<'a> {
    scope_class: &'a str,
    vm_lower: String,
    field_pascal: String,
    field_name: &'a str,
    field_type: &'a str,
}

#[derive(Serialize)]
struct ScopeContext<'a> {
    scope_class: &'a str,
    inherited_class: &'a str,
    view_model_class: &'a str,
    args_type: &'a str,
    aspect_class: &'a str,
}

#[derive(Serialize)]
struct InheritedContext<'a> {
    inherited_class: &'a str,
    view_model_class: &'a str,
    state_type: &'a str,
    aspect_class: &'a str,
}

#[derive(Serialize)]
struct ListenerContext<'a> {
    listener_class: &'a str,
    listener_state_class: &'a str,
    scope_class: &'a str,
    view_model_class: &'a str,
}

#[derive(Serialize)]
struct ExtensionContext<'a> {
    extension_class: &'a str,
    proxy_class: &'a str,
    watch_name: &'a str,
    view_model_class: &'a str,
    read_name: &'a str,
    scope_class: &'a str,
}

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
    let base = render_base(
        &generated_base,
        &class.name,
        state_type,
        args_type,
        &initial_state,
    );
    let aspect = render_aspect_class(&aspect_class, &class.name, state_type, state_fields);
    let proxy = render_proxy(
        &proxy_class,
        &scope_class,
        &class.name,
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
            aspect,
            base,
            proxy,
            scope,
            inherited,
            listener,
            extension,
        },
    )
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
            render_template(
                "aspect_selector",
                include_str!("templates/aspect_selector.jinja"),
                AspectSelectorContext {
                    aspect_class,
                    state_type,
                    vm_lower: lower_camel(view_model_class),
                    field_pascal: pascal_case(&field.name),
                    field_name: &field.name,
                    field_type: &field.type_source,
                },
            )
        })
        .collect::<Vec<_>>()
        .join("\n\n");
    [
        render_template(
            "aspect_class",
            include_str!("templates/aspect_class.jinja"),
            AspectContext {
                aspect_class,
                state_type,
            },
        ),
        field_selectors,
    ]
    .into_iter()
    .filter(|part| !part.is_empty())
    .collect::<Vec<_>>()
    .join("\n\n")
}

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

fn render_proxy(
    proxy_class: &str,
    scope_class: &str,
    view_model_class: &str,
    state_type: &str,
    aspect_class: &str,
    selector_prefix: &str,
    state_fields: &[StateFieldSpec],
) -> String {
    let field_getters = state_fields
        .iter()
        .map(|field| {
            render_template(
                "proxy_field_getter",
                include_str!("templates/proxy_field_getter.jinja"),
                ProxyFieldContext {
                    scope_class,
                    vm_lower: lower_camel(selector_prefix),
                    field_pascal: pascal_case(&field.name),
                    field_name: &field.name,
                    field_type: &field.type_source,
                },
            )
        })
        .collect::<String>();
    render_template(
        "proxy_class",
        include_str!("templates/proxy_class.jinja"),
        ProxyContext {
            proxy_class,
            scope_class,
            view_model_class,
            state_type,
            aspect_class,
            field_getters,
        },
    )
}

fn render_scope(
    scope_class: &str,
    inherited_class: &str,
    view_model_class: &str,
    args_type: &str,
    aspect_class: &str,
) -> String {
    render_template(
        "scope_class",
        include_str!("templates/scope_class.jinja"),
        ScopeContext {
            scope_class,
            inherited_class,
            view_model_class,
            args_type,
            aspect_class,
        },
    )
}

fn render_inherited(
    inherited_class: &str,
    view_model_class: &str,
    state_type: &str,
    aspect_class: &str,
) -> String {
    render_template(
        "inherited_class",
        include_str!("templates/inherited_class.jinja"),
        InheritedContext {
            inherited_class,
            view_model_class,
            state_type,
            aspect_class,
        },
    )
}

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
