import init from './ship.js';

init().catch((error) => {
    if (!error.message.startsWith("Using exceptions for control flow")) {
        console.error(error);
    }
});
