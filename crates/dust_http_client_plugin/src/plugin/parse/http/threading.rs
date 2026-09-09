//! Parsing the threading configuration an HTTP client annotation carries.

use super::*;

/// Parses an `HttpParseThread` enum option from class or method config.
pub(super) fn parse_thread_config(
    config: &ConfigApplicationIr,
    diagnostics: &mut Vec<Diagnostic>,
) -> ParseThreadMode {
    let thread = config
        .named_member("parseThread")
        .or_else(|| config.named_member("thread"));
    match thread.as_deref() {
        Some("main") | Some("HttpParseThread.main") => ParseThreadMode::Main,
        Some("isolate") | Some("HttpParseThread.isolate") => ParseThreadMode::Isolate,
        _ => {
            diagnostics.push(
                Diagnostic::error(
                    "`parseThread` must be `HttpParseThread.main` or `HttpParseThread.isolate`",
                )
                .with_label(label(
                    config.span,
                    "pick one of the supported parse-thread enum values",
                )),
            );
            ParseThreadMode::Main
        }
    }
}
