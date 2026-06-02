use std::{fs, sync::Arc};

use dust_plugin_api::{DustPlugin, SymbolPlan, WorkspaceAnalysisBuilder};
use dust_state_plugin::register_plugin;

use super::support::temp_root;
use crate::support::{args_class, library_with_classes, view_model_class};

#[test]
fn emits_state_fields_from_imported_unannotated_state_file() {
    let root = temp_root("imported_state_fields");
    fs::create_dir_all(root.join("lib/models")).unwrap();
    fs::write(
        root.join("lib/models/products_state.dart"),
        "enum ProductsStatus { initial, success }\n\
         class ProductsState {\n\
           final List<Product> products;\n\
           final ProductsStatus status;\n\
           final String? errorMessage;\n\
           const ProductsState({this.products = const [], required this.status, this.errorMessage});\n\
         }\n",
    )
    .unwrap();

    let plugin = register_plugin();
    let mut library = library_with_classes(vec![
        args_class(),
        view_model_class(
            "ProductsViewModel",
            "(state: ProductsState, args: TaskBoardArgs)",
        ),
    ]);
    library.package_root = root.display().to_string();
    library.source_path = "lib/view_models/products_view_model.dart".to_owned();
    library
        .imports
        .push("../models/products_state.dart".to_owned());

    let contribution = plugin.emit(&library, &SymbolPlan::default());
    let source = &contribution.support_types[0];

    assert!(source.contains("final class _ProductsViewModelAspect<R>"));
    assert!(source.contains("final _productsViewModelProductsAspect"));
    assert!(source.contains("final _productsViewModelStatusAspect"));
    assert!(source.contains("final _productsViewModelErrorMessageAspect"));
    assert!(source.contains("List<Object?> get products"));
    assert!(source.contains("ProductsStatus get status"));
    assert!(source.contains("String? get errorMessage"));

    let _ = fs::remove_dir_all(root);
}

#[test]
fn imported_state_source_wins_over_raw_workspace_field_facts() {
    let root = temp_root("imported_state_precedence");
    fs::create_dir_all(root.join("lib/models")).unwrap();
    fs::write(
        root.join("lib/models/products_state.dart"),
        "import 'product.dart';\n\
         enum ProductsStatus { initial, success }\n\
         class ProductsState {\n\
           final List<Product> products;\n\
           final ProductsStatus status;\n\
           const ProductsState({this.products = const [], required this.status});\n\
         }\n",
    )
    .unwrap();

    let plugin = register_plugin();
    let mut builder = WorkspaceAnalysisBuilder::default();
    builder.add_string_set_value(
        "dust_state.states.v1",
        r#"{"class_name":"ProductsState","fields":[{"name":"products","type_source":"List<Product>"},{"name":"status","type_source":"ProductsStatus"}]}"#,
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
    library.package_root = root.display().to_string();
    library.source_path = "lib/view_models/products_view_model.dart".to_owned();
    library
        .imports
        .push("../models/products_state.dart".to_owned());

    let contribution = plugin.emit(&library, &plan);
    let source = &contribution.support_types[0];

    assert!(source.contains("final class _ProductsViewModelAspect<R>"));
    assert!(source.contains("final _productsViewModelProductsAspect"));
    assert!(source.contains("final _productsViewModelStatusAspect"));
    assert!(source.contains("List<Object?> get products"));
    assert!(source.contains("ProductsStatus get status"));
    assert!(!source.contains("List<Product> get products"));

    let _ = fs::remove_dir_all(root);
}
