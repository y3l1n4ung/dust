pub(crate) fn available_worker_count(item_count: usize, limit: Option<usize>) -> usize {
    if item_count == 0 {
        return 1;
    }

    let max_workers = std::thread::available_parallelism()
        .map(usize::from)
        .unwrap_or(1);
    limit.unwrap_or(max_workers).max(1).min(item_count)
}

pub(crate) fn round_robin_groups<T>(
    items: impl IntoIterator<Item = T>,
    groups: usize,
) -> Vec<Vec<T>> {
    let groups = groups.max(1);
    let mut grouped = (0..groups).map(|_| Vec::new()).collect::<Vec<_>>();

    for (index, item) in items.into_iter().enumerate() {
        grouped[index % groups].push(item);
    }

    grouped
}

#[cfg(test)]
mod tests {
    use super::{available_worker_count, round_robin_groups};

    #[test]
    fn worker_count_caps_to_item_count() {
        assert_eq!(available_worker_count(0, None), 1);
        assert_eq!(available_worker_count(2, Some(8)), 2);
        assert_eq!(available_worker_count(2, Some(1)), 1);
    }

    #[test]
    fn round_robin_groups_preserve_distribution_order() {
        let groups = round_robin_groups(0..7, 3);
        assert_eq!(groups, vec![vec![0, 3, 6], vec![1, 4], vec![2, 5]]);
    }
}
