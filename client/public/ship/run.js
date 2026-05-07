import init from './ship.js';

const canvas = document.getElementById('ship-game-canvas');
if (canvas) {
    canvas.focus();
    canvas.addEventListener('pointerdown', () => canvas.focus());
}

canvas?.addEventListener('keydown', (event) => {
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
