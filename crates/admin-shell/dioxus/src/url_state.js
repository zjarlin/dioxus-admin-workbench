function createApplicationUrlState(window) {
  const document = window.document;
  const listeners = new Set();
  const scopes = new Map();
  const views = new Map();
  let scope = null;
  let batch = null;
  let restore = null;
  let scrollTimer = null;
  const pendingScroll = new Map();
  const previousScrollRestoration = window.history.scrollRestoration;
  const owned = name => name.startsWith('view.') || name === 'route' || name === 'scroll';
  const active = url => url.searchParams.get('account') || url.searchParams.get('page');
  const current = () => new URL(window.location.href);
  const snapshot = () => {
    const url = current();
    return { page: url.searchParams.get('page'), account: url.searchParams.get('account'),
      parameters: Object.fromEntries(url.searchParams) };
  };
  const emit = () => {
    const value = snapshot();
    for (const listener of listeners) listener(value);
    window.dispatchEvent(new window.CustomEvent('admin-url-state', { detail: value }));
  };
  const view = url => [...url.searchParams].filter(([name]) => owned(name));
  const remember = url => {
    if (active(url)) views.set(active(url), view(url));
  };
  const clearView = url => {
    for (const name of [...url.searchParams.keys()]) {
      if (owned(name)) url.searchParams.delete(name);
    }
  };
  const write = (url, mode, group = '') => {
    if (url.href === window.location.href) return false;
    clearTimeout(scrollTimer);
    scrollTimer = null;
    pendingScroll.clear();
    const now = Date.now();
    const replace = mode === 'replace' || (mode === 'continuous' && batch?.group === group && now - batch.time < 700);
    batch = mode === 'continuous' ? { group, time: now } : null;
    window.history[replace ? 'replaceState' : 'pushState'](window.history.state, '', url.href);
    remember(url);
    emit();
    return true;
  };
  const parseScroll = value => {
    const positions = new Map();
    for (const part of (value || '').split(';').slice(0, 16)) {
      const match = /^([a-zA-Z0-9_-]{1,40}):(\d{1,8}):(\d{1,8})$/.exec(part);
      if (match) positions.set(match[1], { x: Number(match[2]), y: Number(match[3]) });
    }
    return positions;
  };
  const scrollValue = positions => [...positions].sort(([a], [b]) => a.localeCompare(b))
    .filter(([, point]) => point.x || point.y).map(([key, point]) => `${key}:${point.x}:${point.y}`).join(';');
  const pageElement = () => document.querySelector('[data-aio-page-active="true"]');
  const scrollElement = key => {
    const page = pageElement();
    if (key === 'window') return document.scrollingElement;
    if (key === 'content') return page?.closest('.application-shell__content');
    return page?.querySelector(`[data-url-scroll="${key}"]`);
  };
  const restoreScroll = () => {
    const url = current();
    const positions = parseScroll(url.searchParams.get('scroll'));
    restore = { page: active(url), positions, deadline: Date.now() + 15000 };
    window.requestAnimationFrame(applyScroll);
  };
  function applyScroll() {
    if (!restore || active(current()) !== restore.page) return;
    if (Date.now() > restore.deadline) { restore = null; return; }
    const page = pageElement();
    if (page?.dataset.aioPage !== restore.page) return;
    if (!restore.positions.has('window')) restore.positions.set('window', { x: 0, y: 0 });
    if (!restore.positions.has('content')) restore.positions.set('content', { x: 0, y: 0 });
    for (const node of page.querySelectorAll('[data-url-scroll]')) {
      if (!restore.positions.has(node.dataset.urlScroll)) restore.positions.set(node.dataset.urlScroll, { x: 0, y: 0 });
    }
    let pending = false;
    for (const [key, point] of restore.positions) {
      if (key.startsWith('guest-')) continue;
      const element = scrollElement(key);
      if (!element && (key === 'content' || key === 'window')) continue;
      if (!element || element.scrollHeight < point.y + element.clientHeight || element.scrollWidth < point.x + element.clientWidth) {
        pending = true;
        continue;
      }
      element.scrollLeft = point.x;
      element.scrollTop = point.y;
    }
    if (!pending) {
      const completed = restore;
      window.requestAnimationFrame(() => { if (restore === completed) restore = null; });
    }
  }
  const observer = new window.MutationObserver(() => { if (restore) window.requestAnimationFrame(applyScroll); });
  observer.observe(document.documentElement, { childList: true, subtree: true, attributes: true,
    attributeFilter: ['data-aio-page-active'] });
  const recordScroll = ({ page, key, x, y }) => {
    if (page !== active(current()) || !/^[a-zA-Z0-9_-]{1,40}$/.test(key) || !Number.isFinite(x) || !Number.isFinite(y)) return;
    const url = current();
    const positions = parseScroll(url.searchParams.get('scroll'));
    if (positions.size >= 16 && !positions.has(key)) return;
    positions.set(key, { x: Math.max(0, Math.min(99999999, Math.round(x))), y: Math.max(0, Math.min(99999999, Math.round(y))) });
    const value = scrollValue(positions);
    if (value) url.searchParams.set('scroll', value);
    else url.searchParams.delete('scroll');
    // 滚动只更新当前节点，不通知页面重新查询，也不打断正在合并的输入。
    if (url.href !== window.location.href) {
      window.history.replaceState(window.history.state, '', url.href);
      remember(url);
    }
  };
  const scrolled = event => {
    if (restore && Date.now() <= restore.deadline) return;
    restore = null;
    const element = event.target === document ? document.scrollingElement : event.target;
    const page = pageElement();
    if (!page || !element) return;
    let key = element.dataset?.urlScroll;
    if (element === document.scrollingElement) key = 'window';
    else if (element === page.closest('.application-shell__content')) key = 'content';
    else if (!page.contains(element)) return;
    if (!key) return;
    pendingScroll.set(key, { page: page.dataset.aioPage, key, x: element.scrollLeft, y: element.scrollTop });
    clearTimeout(scrollTimer);
    scrollTimer = setTimeout(flushScroll, 120);
  };
  const flushScroll = () => {
    for (const value of pendingScroll.values()) recordScroll(value);
    pendingScroll.clear();
    clearTimeout(scrollTimer);
    scrollTimer = null;
  };
  const restoreHistory = () => {
    clearTimeout(scrollTimer);
    scrollTimer = null;
    pendingScroll.clear();
    batch = null;
    remember(current());
    restoreScroll();
    emit();
  };
  window.history.scrollRestoration = 'manual';
  window.addEventListener('popstate', restoreHistory);
  document.addEventListener('scroll', scrolled, { capture: true, passive: true });
  window.addEventListener('resize', applyScroll);
  const cancelRestore = () => { restore = null; };
  for (const name of ['wheel', 'touchstart', 'pointerdown']) window.addEventListener(name, cancelRestore, { passive: true });
  restoreScroll();
  const controller = {
    subscribe(listener) {
      listeners.add(listener);
      listener(snapshot());
      return () => { listeners.delete(listener); if (!listeners.size) controller.dispose(); };
    },
    dispose() {
      flushScroll();
      restore = null;
      observer.disconnect();
      window.removeEventListener('popstate', restoreHistory);
      window.removeEventListener('resize', applyScroll);
      document.removeEventListener('scroll', scrolled, true);
      for (const name of ['wheel', 'touchstart', 'pointerdown']) window.removeEventListener(name, cancelRestore);
      window.history.scrollRestoration = previousScrollRestoration;
      listeners.clear(); scopes.clear(); views.clear();
      if (window.__adminUrlState === controller) delete window.__adminUrlState;
    },
    navigate({ page, account = null, mode = 'push' }) {
      flushScroll();
      const url = current();
      const destination = account || page;
      if (destination !== active(url)) {
        remember(url);
        clearView(url);
        for (const [name, value] of views.get(destination) || []) url.searchParams.set(name, value);
      }
      if (page) url.searchParams.set('page', page); else url.searchParams.delete('page');
      if (account) url.searchParams.set('account', account); else url.searchParams.delete('account');
      if (write(url, mode)) restoreScroll();
    },
    update({ page, values, mode = 'push' }) {
      flushScroll();
      const url = current();
      if (page !== active(url)) return;
      let changed = false;
      for (const [name, value] of Object.entries(values)) {
        if (!owned(name) || name === 'scroll') continue;
        if (value === null || value === '') {
          changed ||= url.searchParams.has(name);
          url.searchParams.delete(name);
        } else if (typeof value === 'string' && value.length <= 4096) {
          changed ||= url.searchParams.get(name) !== value || url.searchParams.getAll(name).length !== 1;
          url.searchParams.set(name, value);
        }
      }
      if (!changed) return;
      if ([...Object.keys(values)].some(name => owned(name) && name !== 'scroll')) url.searchParams.delete('scroll');
      if (write(url, mode, `${page}:${Object.keys(values).sort().join(',')}`)) {
        restoreScroll();
      }
    },
    parameter(page, name) { return active(current()) === page ? current().searchParams.get(name) : new Map(views.get(page) || []).get(name); },
    recordScroll,
    scroll(page) { return active(current()) === page ? current().searchParams.get('scroll') || '' : ''; },
    setScope({ scope: next, page }) {
      if (scope === next) return;
      if (scope === null) { scope = next; return; }
      flushScroll();
      remember(current());
      scopes.set(scope, { page: current().searchParams.get('page'), views: new Map(views) });
      scope = next;
      const saved = scopes.get(next);
      views.clear();
      for (const [id, state] of saved?.views || []) views.set(id, state);
      const url = current();
      clearView(url);
      url.searchParams.delete('account');
      const selected = saved?.page || page;
      if (selected) url.searchParams.set('page', selected); else url.searchParams.delete('page');
      for (const [name, value] of views.get(selected) || []) url.searchParams.set(name, value);
      write(url, 'replace');
      restoreScroll();
    },
  };
  return controller;
}
