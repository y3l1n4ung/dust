#![cfg(test)]

use std::collections::HashMap;

use super::inheritance::merged_fields_for_class;
use dust_ir::{ClassIr, ClassKindIr, FieldIr, SpanIr, TraitApplicationIr, TypeIr};
use dust_text::{FileId, TextRange};

fn span(start: u32, end: u32) -> SpanIr {
    SpanIr::new(FileId::new(99), TextRange::new(start, end))
}

#[test]
fn merges_inherited_fields_before_own_fields() {
    let classes = vec![
        ClassIr {
            kind: ClassKindIr::Class,
            name: "Entity".to_owned(),
            is_abstract: true,
            superclass_name: None,
            span: span(0, 20),
            fields: vec![FieldIr {
                name: "id".to_owned(),
                ty: TypeIr::string(),
                span: span(1, 2),
                has_default: false,
                serde: None,
            }],
            constructors: Vec::new(),
            traits: Vec::<TraitApplicationIr>::new(),
            serde: None,
        },
        ClassIr {
            kind: ClassKindIr::Class,
            name: "DetailedEntity".to_owned(),
            is_abstract: false,
            superclass_name: Some("Entity".to_owned()),
            span: span(20, 40),
            fields: vec![FieldIr {
                name: "label".to_owned(),
                ty: TypeIr::string(),
                span: span(21, 22),
                has_default: false,
                serde: None,
            }],
            constructors: Vec::new(),
            traits: Vec::<TraitApplicationIr>::new(),
            serde: None,
        },
    ];
    let index_by_name = classes
        .iter()
        .enumerate()
        .map(|(index, class)| (class.name.clone(), index))
        .collect::<HashMap<_, _>>();
    let mut cache = HashMap::new();
    let mut active_stack = Vec::new();
    let mut diagnostics = Vec::new();

    let merged = merged_fields_for_class(
        1,
        &classes,
        &index_by_name,
        &mut cache,
        &mut active_stack,
        &mut diagnostics,
    );

    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(
        merged
            .iter()
            .map(|field| field.name.as_str())
            .collect::<Vec<_>>(),
        vec!["id", "label"]
    );
}
