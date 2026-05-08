use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, ClassKindIr, ParamKind};

use crate::plugin::constants::{GENERATE_TEST, HTTP_CLIENT};
use crate::plugin::model::ReturnMode;
use crate::plugin::parse::{has_config_named, parse_http_client_config};
use crate::plugin::util::{config_name, has_import, label, type_name_is};
use crate::plugin::validate::validate_endpoint;

pub(crate) fn validate_client_class(imports: &[String], class: &ClassIr) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    let wants_http_client = has_config_named(&class.configs, HTTP_CLIENT);
    let wants_generate_test = has_config_named(&class.configs, GENERATE_TEST);

    if !wants_http_client && !wants_generate_test {
        return diagnostics;
    }

    if !wants_http_client && wants_generate_test {
        diagnostics.push(
            Diagnostic::error(format!(
                "`@GenerateTest()` requires `@HttpClient()` on class `{}`",
                class.name
            ))
            .with_label(label(
                class.span,
                "add `@HttpClient()` to generate an HTTP client test surface",
            )),
        );
        return diagnostics;
    }

    if !matches!(class.kind, ClassKindIr::Class) || !class.is_abstract || !class.is_interface {
        diagnostics.push(
            Diagnostic::error(format!(
                "`@HttpClient()` requires an `abstract interface class`, but `{}` does not match that shape",
                class.name
            ))
            .with_label(label(
                class.span,
                "convert this declaration to `abstract interface class`",
            )),
        );
    }

    validate_class_level_configs(class, &mut diagnostics);
    validate_factory_constructor(class, &mut diagnostics);
    for method in &class.methods {
        diagnostics.extend(validate_endpoint(imports, class, method));
    }
    diagnostics
}

pub(crate) fn validate_text_stream_import(
    imports: &[String],
    class: &ClassIr,
    method: &dust_ir::MethodIr,
    mode: ReturnMode,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if mode == ReturnMode::TextStream
        && !has_import(imports, "dart:convert")
        && !has_import(
            imports,
            "package:dust_http_client_annotation/dust_http_client_annotation.dart",
        )
    {
        diagnostics.push(
            Diagnostic::error(format!(
                "method `{}` on `{}` returns `Stream<String>` and requires either `import 'dart:convert';` or the Dust HttpClient annotation package import",
                method.name, class.name
            ))
            .with_label(label(
                method.span,
                "import `package:dust_http_client_annotation/dust_http_client_annotation.dart` or add `import 'dart:convert';` to use generated UTF-8 text stream decoding",
            )),
        );
    }
}

fn validate_class_level_configs(class: &ClassIr, diagnostics: &mut Vec<Diagnostic>) {
    for config in &class.configs {
        match config_name(&config.symbol.0) {
            HTTP_CLIENT => {
                let _ = parse_http_client_config(config, diagnostics);
            }
            GENERATE_TEST => {}
            other => diagnostics.push(
                Diagnostic::error(format!(
                    "`@{other}` is not supported on `@HttpClient()` classes like `{}`",
                    class.name
                ))
                .with_label(label(config.span, "remove this class-level annotation")),
            ),
        }
    }
}

fn validate_factory_constructor(class: &ClassIr, diagnostics: &mut Vec<Diagnostic>) {
    let expected_target = format!("_${}", class.name);
    let factory = class
        .constructors
        .iter()
        .find(|ctor| ctor.name.is_none() && ctor.is_factory);

    let Some(factory) = factory else {
        diagnostics.push(
            Diagnostic::error(format!(
                "`@HttpClient()` requires `factory {}(Dio dio, {{String? baseUrl}}) = {};`",
                class.name, expected_target
            ))
            .with_label(label(
                class.span,
                "add an unnamed redirecting factory constructor for the generated client",
            )),
        );
        return;
    };

    if factory.redirected_target_name.as_deref() != Some(expected_target.as_str()) {
        diagnostics.push(
            Diagnostic::error(format!(
                "factory constructor on `{}` must redirect to `{}`",
                class.name, expected_target
            ))
            .with_label(label(
                factory.span,
                "update the factory redirection target to the generated client class",
            )),
        );
    }

    let Some(dio_param) = factory.params.first() else {
        diagnostics.push(
            Diagnostic::error(format!(
                "factory constructor on `{}` must accept a `Dio` parameter",
                class.name
            ))
            .with_label(label(
                factory.span,
                "expected `Dio dio` as the first parameter",
            )),
        );
        return;
    };

    if !type_name_is(&dio_param.ty, "Dio") || dio_param.kind != ParamKind::Positional {
        diagnostics.push(
            Diagnostic::error(format!(
                "factory constructor on `{}` must start with positional `Dio dio`",
                class.name
            ))
            .with_label(label(
                dio_param.span,
                "change this parameter to a positional `Dio` transport instance",
            )),
        );
    }

    for param in factory.params.iter().skip(1) {
        let valid_name = param.name == "baseUrl" && param.kind == ParamKind::Named;
        let valid_type = param.ty.is_named("String") && param.ty.is_nullable();
        let valid_default = !param.has_default;
        if !(valid_name && valid_type && valid_default) {
            diagnostics.push(
                Diagnostic::error(format!(
                    "factory constructor on `{}` only supports optional named `String? baseUrl` without a default after the `Dio` parameter",
                    class.name
                ))
                .with_label(label(
                    param.span,
                    "remove or rename this parameter to `baseUrl` with nullable type `String?` and no default value",
                )),
            );
        }
    }
}
