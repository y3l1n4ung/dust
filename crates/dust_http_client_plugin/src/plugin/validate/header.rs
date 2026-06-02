use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, ConfigApplicationIr, MethodIr};

use crate::plugin::constants::{HEADERS, HTTP_CLIENT};
use crate::plugin::parse::{parse_headers_config, parse_http_client_headers};
use crate::plugin::util::{config_name, label};
use crate::plugin::validate::param::ParamState;

impl ParamState {
    pub(super) fn record_static_headers(
        &mut self,
        class: &ClassIr,
        method: &MethodIr,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for config in &class.configs {
            if config_name(&config.symbol.0) != HTTP_CLIENT {
                continue;
            }
            for (key, _) in class_header_entries(config, diagnostics) {
                record_header_key(self, class, method, config, &key, diagnostics, "client");
            }
        }

        for config in &method.configs {
            if config_name(&config.symbol.0) != HEADERS {
                continue;
            }
            for (key, _) in parse_headers_config(config, diagnostics) {
                record_header_key(self, class, method, config, &key, diagnostics, "method");
            }
        }
    }
}

fn class_header_entries(
    config: &ConfigApplicationIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> Vec<(String, String)> {
    config
        .named_argument_source("headers")
        .map(|value| parse_http_client_headers(value, config, diagnostics))
        .unwrap_or_default()
}

fn record_header_key(
    state: &mut ParamState,
    class: &ClassIr,
    method: &MethodIr,
    config: &ConfigApplicationIr,
    key: &str,
    diagnostics: &mut Vec<Diagnostic>,
    owner: &str,
) {
    if state.header_keys.insert(key.to_owned()) {
        return;
    }
    diagnostics.push(
        Diagnostic::error(format!(
            "method `{}` on `{}` defines duplicate header key `{}`",
            method.name, class.name, key
        ))
        .with_label(label(
            config.span,
            format!("remove the duplicate static header key from this {owner}"),
        )),
    );
}
