const assert = require('node:assert/strict');
const fs = require('node:fs');
const path = require('node:path');
const vm = require('node:vm');
const { test } = require('node:test');

function fixture(address = 'https://example.test/?page=files') {
  const entries = [address];
  let index = 0;
  const window = new EventTarget();
  window.CustomEvent = CustomEvent;
  window.location = { get href() { return entries[index]; } };
  window.history = {
    state: { unrelated: 7 },
    get length() { return entries.length; },
    pushState(state, title, url) { entries.splice(++index); entries.push(url); },
    replaceState(state, title, url) { entries[index] = url; },
    back() { if (index) { index--; window.dispatchEvent(new Event('popstate')); } },
    forward() { if (index + 1 < entries.length) { index++; window.dispatchEvent(new Event('popstate')); } },
  };
  window.document = { documentElement: {}, querySelector() { return null; }, addEventListener() {}, removeEventListener() {} };
  window.MutationObserver = class { observe() {} disconnect() {} };
  window.requestAnimationFrame = () => {};
  const source = fs.readFileSync(path.join(__dirname, '../src/url_state.js'), 'utf8');
  const create = vm.runInNewContext(`${source}\ncreateApplicationUrlState`, { URL, Date, setTimeout, clearTimeout });
  const navigation = create(window);
  return { window, navigation, parameters: () => new URL(window.location.href).searchParams };
}

test('fresh links restore page, query, scroll and guest route without sender storage', () => {
  const f = fixture('https://example.test/?page=files&view.category=application&view.q=%E5%8D%83%E5%AF%BB&scroll=files-list%3A0%3A420&route=%23%2Fdetails%3Ftab%3Dhistory');
  let state;
  f.navigation.subscribe(value => state = value);
  assert.equal(state.page, 'files');
  assert.equal(state.parameters['view.category'], 'application');
  assert.equal(state.parameters['view.q'], '千寻');
  assert.equal(f.navigation.parameter('files', 'route'), '#/details?tab=history');
  assert.equal(f.navigation.scroll('files'), 'files-list:0:420');
});

test('committed navigation and view changes survive back and forward', () => {
  const f = fixture();
  f.navigation.update({ page: 'files', values: { 'view.category': 'application' } });
  f.navigation.navigate({ page: 'users' });
  f.navigation.navigate({ page: 'files' });
  assert.equal(f.parameters().get('view.category'), 'application');
  f.window.history.back();
  assert.equal(f.parameters().get('page'), 'users');
  f.window.history.back();
  assert.equal(f.parameters().get('view.category'), 'application');
  f.window.history.back();
  assert.equal(f.parameters().get('view.category'), null);
  f.window.history.forward();
  assert.equal(f.parameters().get('view.category'), 'application');
});

test('continuous typing retains the pre-input node and does not flood history', () => {
  const f = fixture();
  for (const q of ['a', 'ab', 'abc']) f.navigation.update({ page: 'files', values: { 'view.q': q }, mode: 'continuous' });
  assert.equal(f.window.history.length, 2);
  f.window.history.back();
  assert.equal(f.parameters().get('view.q'), null);
  f.window.history.forward();
  assert.equal(f.parameters().get('view.q'), 'abc');
  f.navigation.update({ page: 'files', values: { 'view.q': 'abcd' }, mode: 'continuous' });
  assert.equal(f.window.history.length, 3);
});

test('no-op updates preserve scroll and unrelated repeated parameters', () => {
  const f = fixture('https://example.test/?page=files&view.sort=name&utm_source=a&external=x&external=y');
  f.navigation.recordScroll({ page: 'files', key: 'files-list', x: 0, y: 320 });
  f.navigation.update({ page: 'files', values: { 'view.sort': 'name' } });
  assert.equal(f.window.history.length, 1);
  assert.equal(f.parameters().get('scroll'), 'files-list:0:320');
  f.navigation.navigate({ page: 'users' });
  assert.deepEqual(f.parameters().getAll('external'), ['x', 'y']);
  assert.equal(f.parameters().get('utm_source'), 'a');
  assert.equal(f.parameters().get('view.sort'), null);
});

test('scroll replaces the current entry, is bounded and does not accept inactive pages', () => {
  const f = fixture();
  for (const y of [10, 100, 999]) f.navigation.recordScroll({ page: 'files', key: 'files-list', x: 0, y });
  assert.equal(f.window.history.length, 1);
  assert.equal(f.parameters().get('scroll'), 'files-list:0:999');
  f.navigation.recordScroll({ page: 'users', key: 'files-list', x: 0, y: 10 });
  f.navigation.recordScroll({ page: 'files', key: 'unsafe"]', x: 0, y: 10 });
  f.navigation.recordScroll({ page: 'files', key: 'files-list', x: 0, y: NaN });
  assert.equal(f.parameters().get('scroll'), 'files-list:0:999');
  f.navigation.recordScroll({ page: 'files', key: 'files-list', x: -8, y: 1e10 });
  assert.equal(f.parameters().get('scroll'), 'files-list:0:99999999');
});

test('account pages preserve the background view and workspace caches remain isolated', () => {
  const f = fixture('https://example.test/?page=files&view.category=application');
  f.navigation.setScope({ scope: 'tenant-a:permissions-a', page: 'files' });
  f.navigation.navigate({ page: 'files', account: 'settings' });
  f.navigation.update({ page: 'settings', values: { 'view.tab': 'profile' } });
  f.navigation.navigate({ page: 'files' });
  assert.equal(f.parameters().get('view.category'), 'application');
  assert.equal(f.parameters().get('view.tab'), null);
  f.navigation.setScope({ scope: 'tenant-b:permissions-b', page: 'files' });
  assert.equal(f.parameters().get('view.category'), null);
  f.navigation.update({ page: 'files', values: { 'view.category': 'image' } });
  f.navigation.setScope({ scope: 'tenant-a:permissions-a', page: 'files' });
  assert.equal(f.parameters().get('view.category'), 'application');
});

test('background pages cannot overwrite the current view or host navigation fields', () => {
  const f = fixture();
  f.navigation.update({ page: 'users', values: { 'view.q': 'hidden' } });
  f.navigation.update({ page: 'files', values: { page: 'users', account: 'settings', worker_pair: 'secret' } });
  assert.equal(f.parameters().get('page'), 'files');
  assert.equal(f.parameters().get('view.q'), null);
  assert.equal(f.window.history.length, 1);
});

test('unsubscribing the last shell releases its global controller and history setting', () => {
  const f = fixture();
  f.window.__adminUrlState = f.navigation;
  const stop = f.navigation.subscribe(() => {});
  stop();
  assert.equal(f.window.__adminUrlState, undefined);
  assert.equal(f.window.history.scrollRestoration, undefined);
});
