/**
 * Main entry point for the Time Helm game client.
 * 
 * Initializes the game with a randomly generated user and starts the game client.
 */

import { GameClient } from './game-client';

/**
 * User information interface.
 */
interface User {
    /** Unique user identifier */
    id: string;
    /** Username for display */
    username: string;
    /** Display name */
    display_name: string;
    /** Optional avatar URL */
    avatar_url: string | null;
}

/**
 * Generate a random username by combining adjectives, nouns, and a number.
 * 
 * @returns A randomly generated username (e.g., "SwiftWolf123")
 */
function generateRandomUsername(): string {
    const adjectives = ['Swift', 'Brave', 'Clever', 'Mighty', 'Wise', 'Bold', 'Calm', 'Bright'];
    const nouns = ['Wolf', 'Eagle', 'Bear', 'Fox', 'Hawk', 'Lion', 'Tiger', 'Dragon'];
    const adjective = adjectives[Math.floor(Math.random() * adjectives.length)];
    const noun = nouns[Math.floor(Math.random() * nouns.length)];
    const number = Math.floor(Math.random() * 1000);
    return `${adjective}${noun}${number}`;
}

/**
 * Initialize the game client.
 * 
 * Generates a random user ID and username, shows the HUD, and starts the game.
 */
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

    // Show HUD (Heads-Up Display)
    document.getElementById('hud')?.classList.add('visible');

    // Start game immediately
    const gameClient = new GameClient(user);
    gameClient.init();
}

// Initialize game when script loads
init();
