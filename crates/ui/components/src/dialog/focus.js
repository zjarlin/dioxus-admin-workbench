const [id, modal] = await dioxus.recv();
const dialog = document.getElementById(id);
if (!dialog) return;
const previous = document.activeElement;
const topmost = () => [...document.querySelectorAll('.dx-dialog')].filter(node => node.getClientRects().length).at(-1) === dialog;
const focusable = () => [...dialog.querySelectorAll('button, input, select, textarea, a[href], [tabindex]')]
  .filter(node => !node.matches(':disabled') && node.tabIndex >= 0 && node.getClientRects().length && !node.closest('[inert]'));
const initial = () => (dialog.querySelector('[autofocus]') || focusable()[0] || dialog).focus();
const trap = event => {
  if (!modal || !topmost() || event.key !== 'Tab') return;
  const items = focusable();
  const active = document.activeElement;
  if (!items.length) { event.preventDefault(); dialog.focus(); return; }
  if (event.shiftKey && (active === items[0] || !items.includes(active))) {
    event.preventDefault(); items.at(-1).focus();
  } else if (!event.shiftKey && (active === items.at(-1) || !items.includes(active))) {
    event.preventDefault(); items[0].focus();
  }
};
const contain = event => { if (modal && topmost() && !dialog.contains(event.target)) initial(); };
document.addEventListener('keydown', trap, true);
document.addEventListener('focusin', contain);
initial();
try { await dioxus.recv(); }
finally {
  document.removeEventListener('keydown', trap, true);
  document.removeEventListener('focusin', contain);
  if (previous?.isConnected) previous.focus();
}
