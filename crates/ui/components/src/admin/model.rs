#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum SortValue {
    Text(String),
    Number(i128),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ListState {
    pub search: String,
    pub sort_key: String,
    pub descending: bool,
    pub page: usize,
    pub page_size: usize,
}

impl Default for ListState {
    fn default() -> Self {
        Self {
            search: String::new(),
            sort_key: String::new(),
            descending: false,
            page: 0,
            page_size: 10,
        }
    }
}

impl ListState {
    pub fn search(&mut self, value: String) {
        self.search = value;
        self.page = 0;
    }

    pub fn sort(&mut self, key: String) {
        self.descending = self.sort_key == key && !self.descending;
        self.sort_key = key;
        self.page = 0;
    }

    pub fn page_count(&self, total: usize) -> usize {
        total.div_ceil(self.page_size.max(1)).max(1)
    }

    pub fn range(&self, total: usize) -> std::ops::Range<usize> {
        let page = self.page.min(self.page_count(total) - 1);
        let start = page * self.page_size.max(1);
        start..(start + self.page_size.max(1)).min(total)
    }

    pub fn query<R: Clone>(
        &self,
        rows: &[R],
        search: impl Fn(&R) -> String,
        sort: impl Fn(&R, &str) -> SortValue,
    ) -> Vec<R> {
        let terms = self
            .search
            .split_whitespace()
            .map(str::to_lowercase)
            .collect::<Vec<_>>();
        let mut result = rows
            .iter()
            .filter(|row| {
                let text = search(row).to_lowercase();
                terms.iter().all(|term| text.contains(term))
            })
            .cloned()
            .collect::<Vec<_>>();
        if !self.sort_key.is_empty() {
            result.sort_by_cached_key(|row| sort(row, &self.sort_key));
            if self.descending {
                result.reverse();
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn query_filters_all_terms_and_sorts_numbers() {
        let mut state = ListState::default();
        state.sort("size".into());
        let rows = vec![
            ("Alpha document", 10),
            ("Alpha image", 2),
            ("Beta document", 1),
        ];
        let search = |row: &(&str, i32)| row.0.to_owned();
        let sort = |row: &(&str, i32), _: &str| SortValue::Number(row.1.into());
        assert_eq!(state.query(&rows, search, sort)[0].1, 1);
        state.search("ALPHA document".into());
        assert_eq!(
            state.query(&rows, search, sort),
            vec![("Alpha document", 10)]
        );
    }
    #[test]
    fn deletion_and_filtering_keep_page_bounds_valid() {
        let mut state = ListState {
            page: 20,
            ..Default::default()
        };
        assert_eq!(state.range(11), 10..11);
        assert_eq!(state.range(0), 0..0);
        state.search("name".into());
        assert_eq!(state.page, 0);
        state.page_size = 0;
        assert_eq!(state.range(2), 0..1);
    }
}
pub type AsyncResult<T> = std::pin::Pin<Box<dyn std::future::Future<Output = Result<T, String>>>>;
