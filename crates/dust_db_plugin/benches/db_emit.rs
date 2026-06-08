use std::time::Instant;

use dust_db_plugin::register_plugin;
use dust_ir::{
    ClassIr, ClassKindIr, ConfigApplicationIr, ConstructorIr, ConstructorParamIr, FieldIr,
    LibraryIr, MethodIr, MethodParamIr, ParamKind, SpanIr, SymbolId, TraitApplicationIr, TypeIr,
};
use dust_plugin_api::{DustPlugin, SymbolPlan};
use dust_text::{FileId, TextRange};

fn span() -> SpanIr {
    SpanIr::new(FileId::new(1), TextRange::new(0_u32, 1_u32))
}

fn config(symbol: &str, args: &str) -> ConfigApplicationIr {
    ConfigApplicationIr {
        symbol: SymbolId::new(symbol),
        arguments_source: Some(args.to_owned()),
        span: span(),
    }
}

fn result_type(ok: TypeIr) -> TypeIr {
    TypeIr::generic(
        "Future",
        vec![TypeIr::generic(
            "Result",
            vec![ok, TypeIr::named("SqlxError")],
        )],
    )
}

fn main() {
    const METHODS: usize = 500;
    const ITERS: usize = 50;
    let row = ClassIr {
        kind: ClassKindIr::Class,
        name: "BenchRow".to_owned(),
        is_abstract: false,
        is_interface: false,
        superclass_name: None,
        span: span(),
        fields: vec![FieldIr {
            name: "id".to_owned(),
            ty: TypeIr::int(),
            span: span(),
            has_default: false,
            serde: None,
            configs: Vec::new(),
        }],
        constructors: vec![ConstructorIr {
            name: None,
            is_factory: false,
            redirected_target_source: None,
            redirected_target_name: None,
            span: span(),
            params: vec![ConstructorParamIr {
                name: "id".to_owned(),
                ty: TypeIr::int(),
                span: span(),
                kind: ParamKind::Named,
                has_default: false,
                default_value_source: None,
            }],
        }],
        methods: Vec::new(),
        traits: vec![TraitApplicationIr {
            symbol: SymbolId::new("dust_dart::FromRow"),
            span: span(),
        }],
        configs: Vec::new(),
        serde: None,
    };
    let dao = ClassIr {
        kind: ClassKindIr::Class,
        name: "BenchDao".to_owned(),
        is_abstract: true,
        is_interface: false,
        superclass_name: None,
        span: span(),
        fields: Vec::new(),
        constructors: vec![ConstructorIr {
            name: None,
            is_factory: true,
            redirected_target_source: Some("_$BenchDao".to_owned()),
            redirected_target_name: Some("_$BenchDao".to_owned()),
            span: span(),
            params: vec![ConstructorParamIr {
                name: "db".to_owned(),
                ty: TypeIr::named("Executor"),
                span: span(),
                kind: ParamKind::Positional,
                has_default: false,
                default_value_source: None,
            }],
        }],
        methods: (0..METHODS)
            .map(|index| MethodIr {
                name: format!("find{index}"),
                is_static: false,
                is_external: false,
                return_type: result_type(TypeIr::named("BenchRow").nullable()),
                has_body: false,
                body_source: None,
                params: vec![MethodParamIr {
                    name: "id".to_owned(),
                    ty: TypeIr::int(),
                    span: span(),
                    kind: ParamKind::Positional,
                    has_default: false,
                    default_value_source: None,
                    traits: Vec::new(),
                    configs: Vec::new(),
                }],
                span: span(),
                traits: Vec::new(),
                configs: vec![config(
                    "dust_dart::Query",
                    &format!("(r'SELECT id FROM bench WHERE id = $1 /* {index} */')"),
                )],
            })
            .collect(),
        traits: Vec::new(),
        configs: vec![config("dust_dart::SqlxDao", "()")],
        serde: None,
    };
    let library = LibraryIr {
        package_root: ".".to_owned(),
        package_name: "bench".to_owned(),
        source_path: "lib/bench.dart".to_owned(),
        output_path: "lib/bench.g.dart".to_owned(),
        imports: Vec::new(),
        span: span(),
        classes: vec![row, dao],
        enums: Vec::new(),
        query_calls: Vec::new(),
    };
    let plugin = register_plugin();
    let started = Instant::now();
    let mut bytes = 0usize;
    for _ in 0..ITERS {
        let contribution = plugin.emit(&library, &SymbolPlan::default());
        bytes += contribution
            .support_types
            .iter()
            .map(String::len)
            .sum::<usize>();
    }
    let elapsed = started.elapsed();
    println!(
        "db_emit methods={METHODS} iterations={ITERS} bytes={bytes} elapsed_ms={}",
        elapsed.as_millis()
    );
}
