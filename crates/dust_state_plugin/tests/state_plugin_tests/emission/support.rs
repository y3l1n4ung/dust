pub(super) fn extract_extension<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker: {marker}"));
    &source[start..]
}

pub(super) fn extract_class<'a>(source: &'a str, marker: &str) -> &'a str {
    let start = source
        .find(marker)
        .unwrap_or_else(|| panic!("missing marker: {marker}"));
    let mut depth = 0_i32;
    let mut saw_body = false;
    for (offset, ch) in source[start..].char_indices() {
        match ch {
            '{' => {
                depth += 1;
                saw_body = true;
            }
            '}' if saw_body => {
                depth -= 1;
                if depth == 0 {
                    return &source[start..start + offset + ch.len_utf8()];
                }
            }
            _ => {}
        }
    }
    panic!("class body did not close: {marker}");
}
