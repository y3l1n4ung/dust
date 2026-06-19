mod render;
mod support;

use std::{borrow::Cow, collections::BTreeMap};

use dust_diagnostics::Diagnostic;
use dust_ir::{ClassIr, DartFileIr, FieldIr, TypeIr};

use crate::features::{
    COPY_WITH_SYMBOL,
    eq_hash::has_trait,
    names::{NameAllocator, library_declaration_names, lower_first},
    writer::{build_constructor_call_multiline, find_clone_constructor},
};

use self::{
    render::{render_copy_with_getter, render_copy_with_support, render_sentinel_helper},
    support::{copy_with_value_expr, needs_copy_with_sentinel},
};

pub(crate) struct CopyWithPlan {
    by_class: BTreeMap<String, CopyWithNames>,
    samples_by_class: BTreeMap<String, CopyWithSample>,
    shared_helpers: Vec<String>,
}

impl CopyWithPlan {
    pub(crate) fn declarations(&self) -> &[String] {
        &self.shared_helpers
    }

    fn names_for(&self, class_name: &str) -> Option<&CopyWithNames> {
        self.by_class.get(class_name)
    }
}

struct CopyWithNames {
    interface_name: String,
    impl_name: String,
    sentinel: Option<CopyWithSentinel>,
    self_name: String,
    then_name: String,
    callback_value_name: String,
    nested_value_names: BTreeMap<String, String>,
}

struct CopyWithSentinel {
    value_name: String,
}

struct CopyWithSample {
    field_name: String,
    nested_value: &'static str,
}

pub(crate) struct CopyWithEmission {
    pub(crate) mixin_member: String,
    pub(crate) support_type: String,
}

pub(crate) fn plan_copy_with(library: &DartFileIr) -> CopyWithPlan {
    let mut allocator = NameAllocator::new(library_declaration_names(library));
    let mut by_class = BTreeMap::new();
    let mut samples_by_class = BTreeMap::new();
    let mut shared_helpers = Vec::new();

    for class in &library.classes {
        if !has_trait(class, COPY_WITH_SYMBOL) || find_clone_constructor(class).is_none() {
            continue;
        }

        let interface_name = allocator.allocate(format!("_${}CopyWith", class.name));
        let impl_name = allocator.allocate(format!("_${}CopyWithImpl", class.name));
        let mut local_allocator =
            NameAllocator::new(class.fields.iter().map(|field| field.name.as_str()));
        let self_name = local_allocator.allocate("_self");
        let then_name = local_allocator.allocate("_then");
        let callback_value_name = local_allocator.allocate("value");
        let nested_value_names = class
            .fields
            .iter()
            .filter(|field| nested_target_type(&field.ty).is_some_and(|(_, nullable)| nullable))
            .map(|field| {
                (
                    field.name.clone(),
                    local_allocator.allocate(format!("{}Value", field.name)),
                )
            })
            .collect::<BTreeMap<_, _>>();

        if let Some(field) = sample_replacement_field(class) {
            samples_by_class.insert(
                class.name.clone(),
                CopyWithSample {
                    field_name: field.name.clone(),
                    nested_value: sample_value(&field.ty, ValueSampleKind::Nested),
                },
            );
        }

        let sentinel = if class
            .fields
            .iter()
            .any(|field| needs_copy_with_sentinel(&field.ty))
        {
            let class_name = allocator.allocate(format!("_{}CopyWithUnset", class.name));
            let value_name =
                allocator.allocate(format!("_{}CopyWithUnset", lower_first(&class.name)));
            shared_helpers.push(render_sentinel_helper(&class_name, &value_name));
            Some(CopyWithSentinel { value_name })
        } else {
            None
        };

        by_class.insert(
            class.name.clone(),
            CopyWithNames {
                interface_name,
                impl_name,
                sentinel,
                self_name,
                then_name,
                callback_value_name,
                nested_value_names,
            },
        );
    }

    CopyWithPlan {
        by_class,
        samples_by_class,
        shared_helpers,
    }
}

pub(crate) fn emit_copy_with(
    class: &ClassIr,
    plan: &CopyWithPlan,
    include_credit: bool,
) -> Option<CopyWithEmission> {
    if !has_trait(class, COPY_WITH_SYMBOL) {
        return None;
    }

    let constructor = find_clone_constructor(class)?;
    let names = plan.names_for(&class.name)?;
    let values = class
        .fields
        .iter()
        .map(|field| {
            (
                field.name.as_str(),
                Cow::Owned(copy_with_value_expr(
                    &field.name,
                    &field.ty,
                    &names.self_name,
                    names
                        .sentinel
                        .as_ref()
                        .map(|sentinel| sentinel.value_name.as_str()),
                )),
            )
        })
        .collect::<Vec<_>>();
    let call = build_constructor_call_multiline(class, constructor, &values)?;

    Some(CopyWithEmission {
        mixin_member: render_copy_with_getter(class, names, plan),
        support_type: render_copy_with_support(class, names, &call, plan, include_credit),
    })
}

pub(crate) fn validate_copy_with(class: &ClassIr) -> Vec<Diagnostic> {
    if !has_trait(class, COPY_WITH_SYMBOL) {
        return Vec::new();
    }

    if class.is_abstract {
        return vec![Diagnostic::error(format!(
            "`CopyWith` cannot target abstract class `{}` because Dust cannot instantiate it",
            class.name
        ))];
    }

    if find_clone_constructor(class).is_some() {
        Vec::new()
    } else {
        vec![Diagnostic::error(format!(
            "`CopyWith` requires a constructor that accepts every field on class `{}`",
            class.name
        ))]
    }
}

fn sample_replacement_field(class: &ClassIr) -> Option<&FieldIr> {
    class.fields.iter().find(|field| {
        !needs_copy_with_sentinel(&field.ty)
            && sample_value(&field.ty, ValueSampleKind::Replacement) != "null"
    })
}

fn nested_target_type(ty: &TypeIr) -> Option<(&str, bool)> {
    let TypeIr::Named {
        name,
        args,
        nullable,
    } = ty
    else {
        return None;
    };

    if !args.is_empty() {
        return None;
    }

    Some((name.as_ref(), *nullable))
}

enum ValueSampleKind {
    Replacement,
    Nested,
}

fn sample_value(ty: &TypeIr, kind: ValueSampleKind) -> &'static str {
    match ty {
        TypeIr::Builtin {
            kind: dust_ir::BuiltinType::String,
            ..
        } => match kind {
            ValueSampleKind::Replacement => "'John'",
            ValueSampleKind::Nested => "'London'",
        },
        TypeIr::Named { name, args, .. } if args.is_empty() && name.as_ref() == "String" => {
            match kind {
                ValueSampleKind::Replacement => "'John'",
                ValueSampleKind::Nested => "'London'",
            }
        }
        TypeIr::Builtin {
            kind: dust_ir::BuiltinType::Bool,
            ..
        } => "true",
        TypeIr::Named { name, args, .. } if args.is_empty() && name.as_ref() == "bool" => "true",
        TypeIr::Builtin {
            kind: dust_ir::BuiltinType::Int,
            ..
        } => "1",
        TypeIr::Named { name, args, .. } if args.is_empty() && name.as_ref() == "int" => "1",
        TypeIr::Builtin {
            kind: dust_ir::BuiltinType::Double,
            ..
        } => "1.0",
        TypeIr::Named { name, args, .. } if args.is_empty() && name.as_ref() == "double" => "1.0",
        _ => "null",
    }
}
