const API_BASE = window.location.origin;

export interface User {
    id: string;
    username: string;
    display_name: string;
    avatar_url: string | null;
}

export class AuthClient {
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
