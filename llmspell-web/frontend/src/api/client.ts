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

import type { ConfigItem, UpdateConfigResponse, Template, TemplateDetails, LaunchTemplateResponse } from './types';

export const api = {
    getHealth: () => fetchJson<HealthResponse>('/health'),
    getSessions: () => fetchJson<Session[]>(`${API_BASE}/sessions`),
    getSession: (id: string) => fetchJson<Session>(`${API_BASE}/sessions/${id}`),

    // Template API
    getTemplates: () => fetchJson<Template[]>(`${API_BASE}/templates`),
    getTemplate: (id: string) => fetchJson<TemplateDetails>(`${API_BASE}/templates/${id}`),
    launchTemplate: (id: string, config: Record<string, any>) => fetchJson<LaunchTemplateResponse>(`${API_BASE}/templates/${id}/launch`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ params: config })
    }),

    // Config API
    getConfig: () => fetchJson<ConfigItem[]>(`${API_BASE}/config`),
    updateConfig: (overrides: Record<string, string>) => fetchJson<UpdateConfigResponse>(`${API_BASE}/config`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ overrides })
    }),
};
