#[derive(Debug, Clone, Copy)]
pub struct Paginator {
    current_page: usize,
    per_page: usize,
    pub maybe_limit: Option<usize>,
    pub maybe_offset: Option<usize>,
    pub maybe_cursor: Option<usize>,
}

impl Default for Paginator {
    fn default() -> Self {
        Self {
            per_page: 15,
            current_page: 0,
            maybe_limit: None,
            maybe_offset: None,
            maybe_cursor: None,
        }
    }
}

impl Paginator {
    fn inner_paginate(&mut self) {
        let per_page = self.per_page;
        let offset = self.current_page * per_page;
        self.maybe_offset = Some(offset);
        self.maybe_limit = Some(per_page);
    }

    pub fn limit(&mut self, limit: usize) {
        self.maybe_limit = Some(limit);
    }

    pub fn offset(&mut self, offset: usize) {
        self.maybe_offset = Some(offset);
    }
}
