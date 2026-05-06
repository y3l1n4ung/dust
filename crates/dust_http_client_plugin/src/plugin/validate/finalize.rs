use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, MethodIr};

use crate::plugin::model::{HttpVerb, RequestMode};
use crate::plugin::parse::method_path;
use crate::plugin::util::{extract_path_placeholders, label};
use crate::plugin::validate::param::ParamState;

impl ParamState {
    pub(super) fn finish(
        &self,
        class: &ClassIr,
        method: &MethodIr,
        verb: HttpVerb,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if self.body_count > 1 {
            diagnostics.push(
                Diagnostic::error(format!(
                    "method `{}` on `{}` can only use one `@Body()` parameter",
                    method.name, class.name
                ))
                .with_label(label(
                    method.span,
                    "remove the extra `@Body()` parameter annotations",
                )),
            );
        }
        if matches!(verb, HttpVerb::Get | HttpVerb::Head | HttpVerb::Options) && self.body_count > 0
        {
            diagnostics.push(
                Diagnostic::error(format!(
                    "method `{}` on `{}` does not allow `@Body()` for `{}` requests",
                    method.name,
                    class.name,
                    verb.as_str()
                ))
                .with_label(label(
                    method.span,
                    "remove the request body from this HTTP verb",
                )),
            );
        }
        if self.request_mode == RequestMode::FormUrlEncoded && self.body_count > 0 {
            diagnostics.push(
                Diagnostic::error(format!(
                    "method `{}` on `{}` cannot combine `@FormUrlEncoded()` with `@Body()`",
                    method.name, class.name
                ))
                .with_label(label(
                    method.span,
                    "use `@Field()` parameters instead of `@Body()` for form requests",
                )),
            );
        }
        if self.request_mode == RequestMode::MultiPart && self.body_count > 0 {
            diagnostics.push(
                Diagnostic::error(format!(
                    "method `{}` on `{}` cannot combine `@MultiPart()` with `@Body()`",
                    method.name, class.name
                ))
                .with_label(label(
                    method.span,
                    "use `@Part()` parameters instead of `@Body()` for multipart requests",
                )),
            );
        }
        if self.request_mode != RequestMode::FormUrlEncoded && !self.field_keys.is_empty() {
            diagnostics.push(
                Diagnostic::error(format!(
                    "method `{}` on `{}` uses `@Field()` without `@FormUrlEncoded()`",
                    method.name, class.name
                ))
                .with_label(label(
                    method.span,
                    "add `@FormUrlEncoded()` to this method or remove the form field annotations",
                )),
            );
        }
        if self.request_mode != RequestMode::MultiPart && !self.part_keys.is_empty() {
            diagnostics.push(
                Diagnostic::error(format!(
                    "method `{}` on `{}` uses `@Part()` without `@MultiPart()`",
                    method.name, class.name
                ))
                .with_label(label(
                    method.span,
                    "add `@MultiPart()` to this method or remove the multipart part annotations",
                )),
            );
        }
        self.finish_special_counts(class, method, diagnostics);
        self.finish_path_placeholders(class, method, diagnostics);
    }

    fn finish_special_counts(
        &self,
        class: &ClassIr,
        method: &MethodIr,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        if self.cancel_token_count > 1 {
            diagnostics.push(
                Diagnostic::error(format!(
                    "method `{}` on `{}` can only declare one `CancelToken` parameter",
                    method.name, class.name
                ))
                .with_label(label(
                    method.span,
                    "keep at most one unannotated `CancelToken` parameter",
                )),
            );
        }
        if self.options_count > 1 {
            diagnostics.push(
                Diagnostic::error(format!(
                    "method `{}` on `{}` can only declare one `Options` parameter",
                    method.name, class.name
                ))
                .with_label(label(
                    method.span,
                    "keep at most one unannotated `Options` parameter",
                )),
            );
        }
        if self.on_send_progress_count > 1 || self.on_receive_progress_count > 1 {
            diagnostics.push(
                Diagnostic::error(format!(
                    "method `{}` on `{}` can only declare one `onSendProgress` and one `onReceiveProgress` callback",
                    method.name, class.name
                ))
                .with_label(label(method.span, "remove the duplicate progress callback parameter")),
            );
        }
    }

    fn finish_path_placeholders(
        &self,
        class: &ClassIr,
        method: &MethodIr,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let path = match method_path(method, diagnostics) {
            Some(path) => path,
            None => return,
        };
        let placeholders = extract_path_placeholders(&path);
        for placeholder in &placeholders {
            if !self.path_keys.contains(placeholder) {
                diagnostics.push(
                    Diagnostic::error(format!(
                        "method `{}` on `{}` is missing `@Path('{}')` for route placeholder `{{{}}}`",
                        method.name, class.name, placeholder, placeholder
                    ))
                    .with_label(label(
                        method.span,
                        "add a matching `@Path()` parameter for every route placeholder",
                    )),
                );
            }
        }
        for key in &self.path_keys {
            if !placeholders.contains(key) {
                diagnostics.push(
                    Diagnostic::error(format!(
                        "method `{}` on `{}` declares `@Path('{}')` that does not appear in the route",
                        method.name, class.name, key
                    ))
                    .with_label(label(
                        method.span,
                        "remove this extra `@Path()` binding or add the placeholder to the route",
                    )),
                );
            }
        }
    }
}
