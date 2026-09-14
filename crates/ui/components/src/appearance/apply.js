const root = document.documentElement;
root.dataset.themeMode = preference.theme;
root.dataset.density = preference.density;
const system = matchMedia('(prefers-color-scheme: dark)');
const update = () => {
  root.dataset.theme = root.dataset.themeMode === 'system'
    ? (system.matches ? 'dark' : 'light') : root.dataset.themeMode;
};
if (!globalThis.__workbenchAppearanceListener) {
  globalThis.__workbenchAppearanceListener = update;
  system.addEventListener('change', update);
}
update();
return true;
