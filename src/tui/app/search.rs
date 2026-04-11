mod input;
mod lists;
mod matchers;

#[derive(Debug, Default)]
pub(crate) struct ListSearch {
    query: String,
    is_editing: bool,
}

impl ListSearch {
    pub(crate) fn is_active(&self) -> bool {
        !self.query.trim().is_empty()
    }

    pub(crate) fn is_editing(&self) -> bool {
        self.is_editing
    }

    pub(crate) fn display_query(&self) -> &str {
        self.query.trim()
    }
}
