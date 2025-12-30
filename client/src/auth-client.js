const API_BASE = window.location.origin;

export class AuthClient {
    async getCurrentUser(sessionId) {
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

