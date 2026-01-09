import { GameClient } from './game-client';

interface User {
    id: string;
    username: string;
    display_name: string;
    avatar_url: string | null;
}

function generateRandomUsername(): string {
    const adjectives = ['Swift', 'Brave', 'Clever', 'Mighty', 'Wise', 'Bold', 'Calm', 'Bright'];
    const nouns = ['Wolf', 'Eagle', 'Bear', 'Fox', 'Hawk', 'Lion', 'Tiger', 'Dragon'];
    const adjective = adjectives[Math.floor(Math.random() * adjectives.length)];
    const noun = nouns[Math.floor(Math.random() * nouns.length)];
    const number = Math.floor(Math.random() * 1000);
    return `${adjective}${noun}${number}`;
}

function init(): void {
    // Generate a random user ID and username
    const userId = `player_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    const username = generateRandomUsername();

    const user: User = {
        id: userId,
        username: username,
        display_name: username,
        avatar_url: null
    };

    // Show HUD
    document.getElementById('hud')?.classList.add('visible');

    // Start game immediately
    const gameClient = new GameClient(user);
    gameClient.init();
}

init();
