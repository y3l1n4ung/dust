use std::collections::BTreeSet;

use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, ConfigApplicationIr, MethodIr, MethodParamIr};

use crate::plugin::build::{EndpointParamKind, special_param_kind};
use crate::plugin::constants::{
    BODY, EXTRA, FIELD, FORM_URL_ENCODED, HEADER, HEADER_MAP, HEADERS, HTTP_PARSE, MULTI_PART,
    PART, PATH, QUERIES, QUERY,
};
use crate::plugin::model::RequestMode;
use crate::plugin::parse::{
    param_source_names, parse_http_parse_config, parse_optional_string_argument,
    parse_required_string_argument,
};
use crate::plugin::util::{config_name, is_string_keyed_map, label};

#[derive(Default)]
pub(super) struct ParamState {
    pub(super) request_mode: RequestMode,
    pub(super) body_count: u32,
    pub(super) path_keys: BTreeSet<String>,
    pub(super) query_keys: BTreeSet<String>,
    pub(super) header_keys: BTreeSet<String>,
    pub(super) field_keys: BTreeSet<String>,
    pub(super) part_keys: BTreeSet<String>,
    pub(super) extra_keys: BTreeSet<String>,
    pub(super) cancel_token_count: u32,
    pub(super) options_count: u32,
    pub(super) on_send_progress_count: u32,
    pub(super) on_receive_progress_count: u32,
}

impl ParamState {
    pub(super) fn apply_method_config(
        &mut self,
        class: &ClassIr,
        method: &MethodIr,
        config: &ConfigApplicationIr,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match config_name(&config.symbol.0) {
            "GET" | "POST" | "PUT" | "PATCH" | "DELETE" | "HEAD" | "OPTIONS" | HEADERS => {}
            HTTP_PARSE => {
                let _ = parse_http_parse_config(config, diagnostics);
            }
            FORM_URL_ENCODED => {
                if self.request_mode != RequestMode::Standard {
                    diagnostics.push(request_mode_error(class, method, config, FORM_URL_ENCODED));
                }
                self.request_mode = RequestMode::FormUrlEncoded;
            }
            MULTI_PART => {
                if self.request_mode != RequestMode::Standard {
                    diagnostics.push(request_mode_error(class, method, config, MULTI_PART));
                }
                self.request_mode = RequestMode::MultiPart;
            }
            other => diagnostics.push(
                Diagnostic::error(format!(
                    "`@{other}` is not supported on method `{}` of `{}`",
                    method.name, class.name
                ))
                .with_label(label(
                    config.span,
                    "move this annotation to a supported location or remove it",
                )),
            ),
        }
    }

    pub(super) fn validate_param(
        &mut self,
        class: &ClassIr,
        method: &MethodIr,
        param: &MethodParamIr,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let sources = param_source_names(param);
        if sources.len() > 1 {
            diagnostics.push(
                Diagnostic::error(format!(
                    "parameter `{}` on `{}.{}` uses more than one HTTP source annotation",
                    param.name, class.name, method.name
                ))
                .with_label(label(
                    param.span,
                    "keep exactly one of `@Path`, `@Query`, `@Header`, `@Body`, `@Field`, `@Part`, or `@Extra` on this parameter",
                )),
            );
            return;
        }

        if sources.is_empty() {
            self.validate_special_param(class, method, param, diagnostics);
            return;
        }

        match sources[0] {
            PATH => record_key(
                parse_optional_string_argument(param, PATH, diagnostics)
                    .unwrap_or_else(|| param.name.clone()),
                &mut self.path_keys,
                class,
                method,
                param,
                "Path",
                diagnostics,
            ),
            QUERY => record_required_key(
                param,
                QUERY,
                &mut self.query_keys,
                class,
                method,
                diagnostics,
            ),
            QUERIES => {
                validate_string_keyed_map(param, class, method, diagnostics, "Queries");
            }
            HEADER_MAP => {
                validate_string_keyed_map(param, class, method, diagnostics, "HeaderMap");
            }
            HEADER => record_required_key(
                param,
                HEADER,
                &mut self.header_keys,
                class,
                method,
                diagnostics,
            ),
            BODY => self.body_count += 1,
            FIELD => record_required_key(
                param,
                FIELD,
                &mut self.field_keys,
                class,
                method,
                diagnostics,
            ),
            PART => {
                record_required_key(param, PART, &mut self.part_keys, class, method, diagnostics)
            }
            EXTRA => record_required_key(
                param,
                EXTRA,
                &mut self.extra_keys,
                class,
                method,
                diagnostics,
            ),
            other => diagnostics.push(
                Diagnostic::error(format!(
                    "`@{other}` is not supported on parameter `{}` of `{}.{}`",
                    param.name, class.name, method.name
                ))
                .with_label(label(
                    param.span,
                    "move this annotation to a supported method or class location",
                )),
            ),
        }
    }

    fn validate_special_param(
        &mut self,
        class: &ClassIr,
        method: &MethodIr,
        param: &MethodParamIr,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        match special_param_kind(param) {
            Some(EndpointParamKind::CancelToken) => self.cancel_token_count += 1,
            Some(EndpointParamKind::Options) => self.options_count += 1,
            Some(EndpointParamKind::OnSendProgress) => self.on_send_progress_count += 1,
            Some(EndpointParamKind::OnReceiveProgress) => self.on_receive_progress_count += 1,
            Some(EndpointParamKind::UnsupportedProgressName) => diagnostics.push(
                Diagnostic::error(format!(
                    "parameter `{}` on `{}.{}` uses `ProgressCallback` but must be named `onSendProgress` or `onReceiveProgress`",
                    param.name, class.name, method.name
                ))
                .with_label(label(
                    param.span,
                    "rename this `ProgressCallback` parameter to `onSendProgress` or `onReceiveProgress`",
                )),
            ),
            None => diagnostics.push(
                Diagnostic::error(format!(
                    "parameter `{}` on `{}.{}` must declare an HTTP source annotation",
                    param.name, class.name, method.name
                ))
                .with_label(label(
                    param.span,
                    "annotate this parameter with one HTTP source or use a supported Dio special parameter type",
                )),
            ),
        }
    }
}

fn validate_string_keyed_map(
    param: &MethodParamIr,
    class: &ClassIr,
    method: &MethodIr,
    diagnostics: &mut Vec<Diagnostic>,
    annotation: &str,
) {
    if is_string_keyed_map(&param.ty) {
        return;
    }
    diagnostics.push(
        Diagnostic::error(format!(
            "parameter `{}` on `{}.{}` uses `@{annotation}()` but must have type `Map<String, ...>`",
            param.name, class.name, method.name
        ))
        .with_label(label(
            param.span,
            format!("change this parameter type to a string-keyed map for `@{annotation}()`"),
        )),
    );
}

fn record_required_key(
    param: &MethodParamIr,
    annotation: &str,
    keys: &mut BTreeSet<String>,
    class: &ClassIr,
    method: &MethodIr,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if let Some(key) = parse_required_string_argument(param, annotation, diagnostics) {
        record_key(key, keys, class, method, param, annotation, diagnostics);
    }
}

fn record_key(
    key: String,
    keys: &mut BTreeSet<String>,
    class: &ClassIr,
    method: &MethodIr,
    param: &MethodParamIr,
    annotation: &str,
    diagnostics: &mut Vec<Diagnostic>,
) {
    if keys.insert(key.clone()) {
        return;
    }
    diagnostics.push(
        Diagnostic::error(format!(
            "method `{}` on `{}` has duplicate `@{annotation}` key `{}`",
            method.name, class.name, key
        ))
        .with_label(label(
            param.span,
            format!("use a unique `@{annotation}` key for this parameter"),
        )),
    );
}

fn request_mode_error(
    class: &ClassIr,
    method: &MethodIr,
    config: &ConfigApplicationIr,
    mode_name: &str,
) -> Diagnostic {
    Diagnostic::error(format!(
        "method `{}` on `{}` cannot combine `@{mode_name}()` with another request-body mode",
        method.name, class.name
    ))
    .with_label(label(
        config.span,
        "pick exactly one body encoding mode for this method",
    ))
}
