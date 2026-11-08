import init, { run } from './ship.js?v=3';

const canvas = document.getElementById('ship-game-canvas');

function isGameKey(code) {
    return (
        code === 'PageUp' ||
        code === 'PageDown' ||
        code === 'Space' ||
        code === 'KeyK' ||
        code === 'KeyW' ||
        code === 'KeyA' ||
        code === 'KeyS' ||
        code === 'KeyD' ||
        code === 'KeyQ' ||
        code === 'KeyE' ||
        code === 'KeyR' ||
        code === 'KeyF' ||
        code === 'KeyZ' ||
        code === 'KeyX'
    );
}

// Winit listens for keys on the canvas. Forward a key that landed elsewhere
// so the first press still reaches the viewer.
function forwardKey(event) {
    if (!canvas || !isGameKey(event.code)) {
        return;
    }
    event.preventDefault();
    if (event.target === canvas) {
        return;
    }
    canvas.focus();
    canvas.dispatchEvent(
        new KeyboardEvent(event.type, {
            key: event.key,
            code: event.code,
            bubbles: true,
            cancelable: true,
            repeat: event.repeat,
        }),
    );
}

window.addEventListener('keydown', forwardKey, true);
window.addEventListener('keyup', forwardKey, true);
window.addEventListener('pointerdown', () => canvas?.focus());

/** Bevy returns from `run()` once the winit loop is spawned. A second call panics. */
let gameLaunchStarted = false;

async function startGame() {
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
        }
        await init();
        run(0);
    } catch (err) {
        gameLaunchStarted = false;
        throw err;
    }
}

startGame().catch((error) => {
    if (!error.message.startsWith('Using exceptions for control flow')) {
        console.error(error);
    }
});
