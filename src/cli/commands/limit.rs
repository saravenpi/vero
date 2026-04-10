pub(super) fn apply_limit<T>(items: Vec<T>, limit: Option<usize>) -> Vec<T> {
    match limit {
        Some(limit) => items.into_iter().take(limit).collect(),
        None => items,
    }
}
