import init from './ship.js';

// Prevent browser/page scrolling shortcuts from stealing game controls.
window.addEventListener('keydown', (event) => {
    if (
        event.code === 'PageUp' ||
        event.code === 'PageDown' ||
        event.code === 'ArrowUp' ||
        event.code === 'ArrowDown' ||
        event.code === 'Space'
    ) {
        event.preventDefault();
    }
}, { passive: false });

init().catch((error) => {
    if (!error.message.startsWith("Using exceptions for control flow")) {
        console.error(error);
    }
});
