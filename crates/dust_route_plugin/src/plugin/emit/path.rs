pub(super) fn is_path_prefix(parent: &str, child: &str) -> bool {
    let parent_segments = route_segments(parent);
    let child_segments = route_segments(child);
    parent_segments.len() < child_segments.len()
        && parent_segments
            .iter()
            .zip(child_segments)
            .all(|(parent, child)| {
                parent == &child || (parent.starts_with(':') && child.starts_with(':'))
            })
}
pub(super) fn route_segments(path: &str) -> Vec<&str> {
    path.split('/')
        .filter(|segment| !segment.is_empty())
        .collect()
}
