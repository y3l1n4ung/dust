use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, MethodIr};

use crate::plugin::build::classify_return_type;
use crate::plugin::parse::method_verbs;
use crate::plugin::util::label;
use crate::plugin::validate::class::validate_text_stream_import;
use crate::plugin::validate::param::ParamState;

pub(crate) fn validate_endpoint(imports: &[String], class: &ClassIr, method: &MethodIr) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let verbs = method_verbs(method);

    if verbs.is_empty() {
        diagnostics.push(
            Diagnostic::error(format!(
                "method `{}` on `{}` must declare exactly one HTTP verb annotation",
                method.name, class.name
            ))
            .with_label(label(
                method.span,
                "add one of `@GET`, `@POST`, `@PUT`, `@PATCH`, `@DELETE`, `@HEAD`, or `@OPTIONS`",
            )),
        );
        return diagnostics;
    }
    if verbs.len() > 1 {
        diagnostics.push(
            Diagnostic::error(format!(
                "method `{}` on `{}` has more than one HTTP verb annotation",
                method.name, class.name
            ))
            .with_label(label(
                method.span,
                "keep exactly one HTTP verb annotation on this method",
            )),
        );
    }
    if method.has_body {
        diagnostics.push(
            Diagnostic::error(format!(
                "method `{}` on `{}` must not declare an implementation body in an `@HttpClient()` interface",
                method.name, class.name
            ))
            .with_label(label(
                method.span,
                "remove the method body and keep only the abstract signature",
            )),
        );
    }
    let return_spec = classify_return_type(&method.return_type);
    if return_spec.is_none() {
        diagnostics.push(
            Diagnostic::error(format!(
                "method `{}` on `{}` must return `Future<T>`, `Future<Response<T>>`, or a supported stream shape",
                method.name, class.name
            ))
            .with_label(label(
                method.span,
                "adjust this return type to one of the supported asynchronous HTTP shapes",
            )),
        );
    } else if let Some(spec) = return_spec {
        validate_text_stream_import(imports, class, method, spec.mode, &mut diagnostics);
    }

    let mut state = ParamState::default();
    state.record_static_headers(class, method, &mut diagnostics);
    for config in &method.configs {
        state.apply_method_config(class, method, config, &mut diagnostics);
    }
    for param in &method.params {
        state.validate_param(class, method, param, &mut diagnostics);
    }
    state.finish(class, method, verbs[0], &mut diagnostics);
    diagnostics
}
