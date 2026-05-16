import init, { run } from './ship.js';

const canvas = document.getElementById('ship-game-canvas');

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

/** Resolves run mode from URL path: `/seacells` or `/3d` (with optional trailing slash). */
function modeFromPathname() {
    let path = window.location.pathname;
    while (path.length > 1 && path.endsWith('/')) {
        path = path.slice(0, -1);
    }
    if (path === '/seacells') {
        return 1;
    }
    if (path === '/3d') {
        return 0;
    }
    return null;
}

/** On WASM, Bevy returns from `run()` immediately after spawning the winit loop; a second `run()` panics with RecreationAttempt. */
let gameLaunchStarted = false;

async function startGame(mode) {
    if (gameLaunchStarted) {
        return;
    }
    gameLaunchStarted = true;

    try {
        const container = document.getElementById('ship-game-container');
        if (container) {
            container.style.display = 'flex';
        }
        if (canvas) {
            canvas.focus();
            canvas.addEventListener('pointerdown', () => canvas.focus());
        }
        await init();
        run(mode);
    } catch (err) {
        gameLaunchStarted = false;
        throw err;
    }
}

const mode = modeFromPathname();
if (mode !== null) {
    startGame(mode).catch((error) => {
        if (!error.message.startsWith('Using exceptions for control flow')) {
            console.error(error);
        }
    });
} else {
    console.error('Time Helm: open /seacells/ or /3d/');
}
