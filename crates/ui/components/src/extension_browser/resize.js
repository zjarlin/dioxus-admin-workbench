const root = document.querySelector('.extension-browser');
const handle = root?.querySelector('.extension-browser__separator');
if (handle && !handle.dataset.bound) {
  handle.dataset.bound = 'true';
  let width = Number(sessionStorage.getItem('aio-extension-sidebar-width')) || 300;
  const set = value => {
    width = Math.max(240, Math.min(420, value));
    root.style.setProperty('--extension-sidebar-width', `${width}px`);
    handle.setAttribute('aria-valuenow', String(width));
    sessionStorage.setItem('aio-extension-sidebar-width', String(width));
  };
  set(width);
  handle.addEventListener('pointerdown', event => {
    event.preventDefault();
    const start = event.clientX, previous = width;
    handle.setPointerCapture(event.pointerId);
    const move = next => set(previous + next.clientX - start);
    const stop = () => {
      handle.removeEventListener('pointermove', move);
      handle.removeEventListener('pointerup', stop);
      handle.removeEventListener('pointercancel', stop);
    };
    handle.addEventListener('pointermove', move);
    handle.addEventListener('pointerup', stop);
    handle.addEventListener('pointercancel', stop);
  });
  handle.addEventListener('keydown', event => {
    if (event.key === 'ArrowLeft' || event.key === 'ArrowRight') {
      event.preventDefault(); set(width + (event.key === 'ArrowLeft' ? -10 : 10));
    }
  });
}
return true;
