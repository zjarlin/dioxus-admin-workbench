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
    pub scope: PageScope,
    used: u64,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(super) struct PageScope {
    pub id: String,
    pub version: String,
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
        scope: &PageScope,
    ) -> Vec<PageInstance> {
        self.entries.retain(|entry| {
            if entry.scope.id == scope.id {
                entry.scope == *scope && available.contains(&entry.source)
            } else {
                // 原生业务页仍读取宿主当前上下文，不能跨租户保留其请求和状态。
                matches!(entry.source, PageSource::Runtime(..))
            }
        });
        self.clock += 1;
        if let Some(source) = available.iter().find(|page| Some(page.id()) == selected) {
            if let Some(entry) = self
                .entries
                .iter_mut()
                .find(|entry| entry.scope == *scope && entry.source == *source)
            {
                entry.used = self.clock;
            } else {
                self.entries.push(PageInstance {
                    serial: self.clock,
                    source: source.clone(),
                    scope: scope.clone(),
                    used: self.clock,
                });
            }
        }
        while self.entries.len() > capacity.max(1) {
            let oldest = self
                .entries
                .iter()
                .enumerate()
                .filter(|(_, entry)| entry.scope != *scope || Some(entry.source.id()) != selected)
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
        let first = cache.reconcile(&pages, Some("a"), 2, &PageScope::default())[0].serial;
        cache.reconcile(&pages, Some("b"), 2, &PageScope::default());
        let restored = cache.reconcile(&pages, Some("a"), 2, &PageScope::default());
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
        cache.reconcile(&pages, Some("a"), 2, &PageScope::default());
        cache.reconcile(&pages, Some("b"), 2, &PageScope::default());
        cache.reconcile(&pages, Some("a"), 2, &PageScope::default());
        let entries = cache.reconcile(&pages, Some("c"), 2, &PageScope::default());
        assert_eq!(
            entries.iter().map(|p| p.source.id()).collect::<Vec<_>>(),
            ["a", "c"]
        );
        let old = entries[0].serial;
        pages[0] = page("a", "2");
        let entries = cache.reconcile(&pages, Some("a"), 2, &PageScope::default());
        assert!(entries.iter().all(|p| p.serial != old));
        pages.retain(|p| p.id() != "c");
        assert_eq!(
            cache
                .reconcile(&pages, None, 2, &PageScope::default())
                .len(),
            1
        );
        assert!(
            cache
                .reconcile(&[], None, 2, &PageScope::default())
                .is_empty()
        );
    }

    #[test]
    fn tenant_instances_are_isolated_retained_and_revalidated() {
        let mut cache = PageCache::default();
        let a = PageScope {
            id: "a".into(),
            version: "permissions-1".into(),
        };
        let b = PageScope {
            id: "b".into(),
            version: "permissions-1".into(),
        };
        let pages = vec![page("counter", "release-1")];
        let first = cache.reconcile(&pages, Some("counter"), 2, &a)[0].serial;
        let both = cache.reconcile(&pages, Some("counter"), 2, &b);
        assert_eq!(both.len(), 2);
        assert_ne!(both[0].serial, both[1].serial);
        let returned = cache.reconcile(&pages, Some("counter"), 2, &a);
        assert_eq!(returned[0].serial, first);
        let revoked = PageScope {
            version: "permissions-2".into(),
            ..a.clone()
        };
        let replaced = cache.reconcile(&pages, Some("counter"), 2, &revoked);
        assert!(replaced.iter().all(|entry| entry.serial != first));
        let removed = cache.reconcile(&[], None, 2, &revoked);
        assert_eq!(removed.len(), 1);
        assert_eq!(removed[0].scope, b);
        let revised = vec![page("counter", "release-2")];
        let previous = removed[0].serial;
        assert!(
            cache
                .reconcile(&revised, Some("counter"), 2, &b)
                .iter()
                .all(|entry| entry.serial != previous)
        );
    }
}
