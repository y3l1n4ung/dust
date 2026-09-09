use std::collections::BTreeMap;

use super::derive_member_names;
use dust_ir::{AnnotationValueIr, NameIr, SpanIr};
use dust_text::{FileId, TextRange};

#[test]
fn derive_members_come_from_structured_constructor_values() {
    let span = SpanIr::new(FileId::new(1), TextRange::new(0_u32, 10_u32));
    let values = vec![AnnotationValueIr::List(vec![
        AnnotationValueIr::Constructor {
            name: NameIr {
                source: "d.ToString".to_owned(),
                short: "ToString".to_owned(),
                prefix: Some("d".to_owned()),
                span,
            },
            positional_args: Vec::new(),
            named_args: BTreeMap::new(),
        },
        AnnotationValueIr::Expression(dust_ir::ExprSourceIr {
            source: "Unknown()".to_owned(),
            span,
        }),
    ])];

    assert_eq!(derive_member_names(&values), ["ToString"]);
}
