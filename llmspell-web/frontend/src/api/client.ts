export interface HealthResponse {
    status: string;
    uptime_seconds: number;
    version: string;
}

export interface Session {
    id: string;
    created_at: number;
    updated_at: number;
    metadata: Record<string, unknown>;
    status: 'active' | 'completed' | 'failed'; // Assuming these statuses for now
}

export interface ApiError {
    message: string;
    code?: string;
}

const API_BASE = '/api';

async function fetchJson<T>(url: string, options?: RequestInit): Promise<T> {
    const response = await fetch(url, options);
    if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.message || `HTTP error! status: ${response.status}`);
    }
    return response.json();
}

export const api = {
    getHealth: () => fetchJson<HealthResponse>('/health'),
    getSessions: () => fetchJson<Session[]>(`${API_BASE}/sessions`),
    getSession: (id: string) => fetchJson<Session>(`${API_BASE}/sessions/${id}`),
};
