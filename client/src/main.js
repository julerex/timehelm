import * as THREE from 'three';
import { GameClient } from './game-client.js';

function generateRandomUsername() {
    const adjectives = ['Swift', 'Brave', 'Clever', 'Mighty', 'Wise', 'Bold', 'Calm', 'Bright'];
    const nouns = ['Wolf', 'Eagle', 'Bear', 'Fox', 'Hawk', 'Lion', 'Tiger', 'Dragon'];
    const adjective = adjectives[Math.floor(Math.random() * adjectives.length)];
    const noun = nouns[Math.floor(Math.random() * nouns.length)];
    const number = Math.floor(Math.random() * 1000);
    return `${adjective}${noun}${number}`;
}

function init() {
    // Generate a random user ID and username
    const userId = `player_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
    const username = generateRandomUsername();
    
    const user = {
        id: userId,
        username: username,
        display_name: username,
        avatar_url: null,
    };
    
    // Hide login screen, show HUD
    document.getElementById('login-screen').style.display = 'none';
    document.getElementById('hud').classList.add('visible');
    
    // Start game immediately
    const gameClient = new GameClient(user);
    gameClient.init();
}

init();

