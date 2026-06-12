mod class_decl;
mod constructors;
mod fields;
mod methods;
mod parse_text;
mod primary;
mod primary_text;

use dust_parser_dart::ParsedClassSurface;
use dust_text::SourceText;
use tree_sitter::Node;

pub(crate) use primary::extract_primary_constructor_classes;

pub(crate) fn extract_classes(root: Node<'_>, source: &SourceText) -> Vec<ParsedClassSurface> {
    let primary_classes = extract_primary_constructor_classes(source);
    let mut classes = primary_classes.clone();
    let mut cursor = root.walk();

    for child in root.children(&mut cursor).filter(|node| node.is_named()) {
        if child.kind() == "class_declaration"
            && primary_classes
                .iter()
                .all(|primary| !ranges_overlap(primary.span, crate::syntax::text_range(child)))
        {
            classes.push(class_decl::extract_class(child, source));
        }
    }

    classes.sort_by_key(|class| class.span.start());
    classes
}

fn ranges_overlap(left: dust_text::TextRange, right: dust_text::TextRange) -> bool {
    left.start() < right.end() && right.start() < left.end()
}
