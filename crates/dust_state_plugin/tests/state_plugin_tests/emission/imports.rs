use std::sync::Arc;

use dust_plugin_api::{DustPlugin, SymbolPlan, WorkspaceAnalysisBuilder};
use dust_state_plugin::register_plugin;

use crate::support::{args_class, library_with_classes, view_model_class};

#[test]
fn imported_state_fields_do_not_generate_proxy_getters() {
    let plugin = register_plugin();
    let mut builder = WorkspaceAnalysisBuilder::default();
    builder.add_string_set_value(
        "dust_state.states.v1",
        r#"{"class_name":"ProductsState","fields":[{"name":"products","type_source":"List<Object?>"},{"name":"status","type_source":"ProductsStatus"},{"name":"errorMessage","type_source":"String?"}]}"#,
    );
    let mut plan = SymbolPlan::default();
    plan.set_workspace_analysis(Arc::new(builder.build()));

    let mut library = library_with_classes(vec![
        args_class(),
        view_model_class(
            "ProductsViewModel",
            "(state: ProductsState, args: TaskBoardArgs)",
        ),
    ]);
    library.source_path = "lib/view_models/products_view_model.dart".to_owned();
    library
        .imports
        .push("../models/products_state.dart".to_owned());

    let contribution = plugin.emit(&library, &plan);
    let source = &contribution.support_types[0];

    assert!(!source.contains("_ProductsViewModelAspect"));
    assert!(!source.contains("_productsViewModelProductsAspect"));
    assert!(!source.contains("_productsViewModelStatusAspect"));
    assert!(!source.contains("_productsViewModelErrorMessageAspect"));
    assert!(!source.contains("List<Object?> get products"));
    assert!(!source.contains("ProductsStatus get status"));
    assert!(!source.contains("String? get errorMessage"));
    assert!(!source.contains("select<R>"));
    assert!(source.contains("ProductsState get value"));
}

#[test]
fn workspace_state_facts_cannot_leak_imported_field_types() {
    let plugin = register_plugin();
    let mut builder = WorkspaceAnalysisBuilder::default();
    builder.add_string_set_value(
        "dust_state.states.v1",
        r#"{"class_name":"ProductsState","fields":[{"name":"products","type_source":"List<Object?>"},{"name":"status","type_source":"ProductsStatus"}]}"#,
    );
    let mut plan = SymbolPlan::default();
    plan.set_workspace_analysis(Arc::new(builder.build()));

    let mut library = library_with_classes(vec![
        args_class(),
        view_model_class(
            "ProductsViewModel",
            "(state: ProductsState, args: TaskBoardArgs)",
        ),
    ]);
    library.source_path = "lib/view_models/products_view_model.dart".to_owned();
    library
        .imports
        .push("../models/products_state.dart".to_owned());

    let contribution = plugin.emit(&library, &plan);
    let source = &contribution.support_types[0];

    assert!(!source.contains("_ProductsViewModelAspect"));
    assert!(!source.contains("_productsViewModelProductsAspect"));
    assert!(!source.contains("_productsViewModelStatusAspect"));
    assert!(!source.contains("List<Object?> get products"));
    assert!(!source.contains("ProductsStatus get status"));
    assert!(!source.contains("List<Product> get products"));
    assert!(source.contains("ProductsState get value"));
}
