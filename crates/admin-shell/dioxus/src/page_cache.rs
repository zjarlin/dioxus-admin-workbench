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
    pub preparing: bool,
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
    prepared: Vec<(PageScope, PageSource)>,
}

impl PageCache {
    pub fn prepare(
        &mut self,
        available: &[PageSource],
        ids: &[String],
        capacity: usize,
        scope: &PageScope,
    ) -> Vec<PageInstance> {
        self.entries.retain(|entry| {
            !entry.preparing
                || (entry.scope == *scope && ids.iter().any(|id| id == entry.source.id()))
        });
        self.prepared
            .retain(|(previous, source)| previous == scope && available.contains(source));
        for id in ids {
            let Some(source @ PageSource::Runtime(..)) =
                available.iter().find(|page| page.id() == id)
            else {
                continue;
            };
            if self
                .entries
                .iter()
                .any(|entry| entry.scope == *scope && entry.source == *source)
                || self
                    .prepared
                    .iter()
                    .any(|(previous, page)| previous == scope && page == source)
                || self.entries.len() >= capacity.max(1)
            {
                continue;
            }
            self.clock += 1;
            self.prepared.push((scope.clone(), source.clone()));
            self.entries.push(PageInstance {
                serial: self.clock,
                source: source.clone(),
                scope: scope.clone(),
                used: 0,
                preparing: true,
            });
        }
        self.entries.clone()
    }

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
                !entry.preparing && matches!(entry.source, PageSource::Runtime(..))
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
                entry.preparing = false;
            } else {
                self.entries.push(PageInstance {
                    serial: self.clock,
                    source: source.clone(),
                    scope: scope.clone(),
                    used: self.clock,
                    preparing: false,
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
    fn prepared_pages_activate_in_place_and_do_not_displace_visited_pages() {
        let mut cache = PageCache::default();
        let scope = PageScope::default();
        let pages = vec![page("a", "1"), page("b", "1"), page("c", "1")];
        cache.reconcile(&pages, Some("a"), 2, &scope);
        let entries = cache.prepare(&pages, &["b".into(), "c".into()], 2, &scope);
        assert_eq!(entries.len(), 2);
        assert!(entries[1].preparing);
        let serial = entries[1].serial;
        let opened = cache.reconcile(&pages, Some("b"), 2, &scope);
        assert_eq!(opened[1].serial, serial);
        assert!(!opened[1].preparing);
        cache.reconcile(&pages, Some("c"), 2, &scope);
        cache.reconcile(&pages, Some("a"), 2, &scope);
        assert!(
            !cache
                .prepare(&pages, &["b".into()], 2, &scope)
                .iter()
                .any(|entry| entry.source.id() == "b")
        );
    }

    #[test]
    fn unused_preparations_are_removed_on_scope_permission_and_version_changes() {
        let mut cache = PageCache::default();
        let scope = PageScope::default();
        let pages = vec![page("a", "1"), page("b", "1")];
        cache.reconcile(&pages, Some("a"), 3, &scope);
        let old = cache.prepare(&pages, &["b".into()], 3, &scope)[1].serial;
        let pages = vec![page("a", "1"), page("b", "2")];
        cache.reconcile(&pages, Some("a"), 3, &scope);
        assert_ne!(
            cache.prepare(&pages, &["b".into()], 3, &scope)[1].serial,
            old
        );
        assert_eq!(cache.prepare(&pages, &[], 3, &scope).len(), 1);
        let other = PageScope {
            id: "other".into(),
            version: "other".into(),
        };
        cache.reconcile(&pages, Some("a"), 3, &other);
        assert!(
            cache
                .prepare(&pages, &[], 3, &other)
                .iter()
                .all(|entry| !entry.preparing)
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
