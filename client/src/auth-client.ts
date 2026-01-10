/**
 * Authentication client for Twitter/X OAuth.
 * 
 * Handles user authentication and session management.
 * Note: Currently not in use as authentication is disabled.
 */

const API_BASE = window.location.origin;

/**
 * User information interface.
 */
export interface User {
    /** Unique user identifier */
    id: string;
    /** Username */
    username: string;
    /** Display name */
    display_name: string;
    /** Optional avatar URL */
    avatar_url: string | null;
}

/**
 * Client for handling authentication operations.
 */
export class AuthClient {
    /**
     * Get the current authenticated user from the server.
     * 
     * @param sessionId - Session identifier from cookie or query parameter
     * @returns User object if authenticated, null otherwise
     */
    async getCurrentUser(sessionId: string): Promise<User | null> {
        try {
            const response = await fetch(`${API_BASE}/auth/me?session=${sessionId}`);
            if (response.ok) {
                return await response.json();
            }
            return null;
        } catch (error) {
            console.error('Failed to get current user:', error);
            return null;
        }
    }
}
