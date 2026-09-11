use crate::{ApplicationPage, ApplicationRuntimePage};

#[derive(Clone, Debug, PartialEq)]
pub(super) enum PageSource {
    Native(ApplicationPage),
    Runtime(ApplicationRuntimePage, String),
}

impl PageSource {
    pub fn id(&self) -> &str {
        match self {
            Self::Native(page) => page.id,
            Self::Runtime(page, _) => &page.id,
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::Native(page) => page.label,
            Self::Runtime(page, _) => &page.label,
        }
    }
}

#[derive(Clone)]
pub(super) struct PageInstance {
    pub serial: u64,
    pub source: PageSource,
    used: u64,
}

#[derive(Default)]
pub(super) struct PageCache {
    entries: Vec<PageInstance>,
    clock: u64,
}

impl PageCache {
    pub fn reconcile(
        &mut self,
        available: &[PageSource],
        selected: Option<&str>,
        capacity: usize,
    ) -> Vec<PageInstance> {
        self.entries
            .retain(|entry| available.contains(&entry.source));
        self.clock += 1;
        if let Some(source) = available.iter().find(|page| Some(page.id()) == selected) {
            if let Some(entry) = self
                .entries
                .iter_mut()
                .find(|entry| entry.source == *source)
            {
                entry.used = self.clock;
            } else {
                self.entries.push(PageInstance {
                    serial: self.clock,
                    source: source.clone(),
                    used: self.clock,
                });
            }
        }
        while self.entries.len() > capacity.max(1) {
            let oldest = self
                .entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| Some(entry.source.id()) != selected)
                .min_by_key(|(_, entry)| entry.used)
                .map(|(index, _)| index);
            if let Some(index) = oldest {
                self.entries.remove(index);
            } else {
                break;
            }
        }
        // 保持 DOM 插入顺序，不能按最近访问排序，否则移动 iframe 会触发浏览器重载。
        self.entries.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page(id: &str, version: &str) -> PageSource {
        PageSource::Runtime(
            ApplicationRuntimePage {
                id: id.into(),
                label: id.into(),
                icon: None,
                scene_id: "workspace".into(),
                scene_label: "Workspace".into(),
                menu_path: vec![],
                required_permission: None,
                definition: "{}".into(),
            },
            version.into(),
        )
    }

    #[test]
    fn revisiting_preserves_instances_and_dom_order() {
        let mut cache = PageCache::default();
        let pages = vec![page("a", "1"), page("b", "1")];
        let first = cache.reconcile(&pages, Some("a"), 2)[0].serial;
        cache.reconcile(&pages, Some("b"), 2);
        let restored = cache.reconcile(&pages, Some("a"), 2);
        assert_eq!(restored[0].serial, first);
        assert_eq!(
            restored.iter().map(|p| p.source.id()).collect::<Vec<_>>(),
            ["a", "b"]
        );
    }

    #[test]
    fn only_inactive_lru_is_evicted_and_versions_are_invalidated() {
        let mut cache = PageCache::default();
        let mut pages = vec![page("a", "1"), page("b", "1"), page("c", "1")];
        cache.reconcile(&pages, Some("a"), 2);
        cache.reconcile(&pages, Some("b"), 2);
        cache.reconcile(&pages, Some("a"), 2);
        let entries = cache.reconcile(&pages, Some("c"), 2);
        assert_eq!(
            entries.iter().map(|p| p.source.id()).collect::<Vec<_>>(),
            ["a", "c"]
        );
        let old = entries[0].serial;
        pages[0] = page("a", "2");
        let entries = cache.reconcile(&pages, Some("a"), 2);
        assert!(entries.iter().all(|p| p.serial != old));
        pages.retain(|p| p.id() != "c");
        assert_eq!(cache.reconcile(&pages, None, 2).len(), 1);
        assert!(cache.reconcile(&[], None, 2).is_empty());
    }
}
