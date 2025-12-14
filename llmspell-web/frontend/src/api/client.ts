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

async function fetchJson<T>(url: string, options: RequestInit = {}): Promise<T> {
    const token = localStorage.getItem('token');
    const headers = {
        ...(options.headers || {}),
        'Authorization': token ? `Bearer ${token}` : '',
        'Content-Type': 'application/json',
    };

    // Remove content-type if body is FormData/undefined to let browser handle it? 
    // Usually for JSON APIs it's fine to force it, but let's be careful.
    // For now, simpler:
    const config = {
        ...options,
        headers: headers as HeadersInit,
    };

    const response = await fetch(url, config);
    if (!response.ok) {
        const errorData = await response.json().catch(() => ({}));
        throw new Error(errorData.message || `HTTP error! status: ${response.status}`);
    }
    return response.json();
}

import type { ConfigItem, UpdateConfigResponse, Template, TemplateDetails, LaunchTemplateResponse, ExecuteScriptRequest, ExecuteScriptResponse, ToolDefinition, ExecuteToolResponse, SessionDetailsResponse } from './types';

interface LoginResponse {
    token: string;
}

export const api = {
    login: (apiKey: string) => fetchJson<LoginResponse>(`${API_BASE}/login`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ api_key: apiKey }),
    }),
    getHealth: () => fetchJson<HealthResponse>('/health'),
    getSessions: () => fetchJson<Session[]>(`${API_BASE}/sessions`),
    getSession: (id: string) => fetchJson<Session>(`${API_BASE}/sessions/${id}`),
    getSessionDetails: (id: string) => fetchJson<SessionDetailsResponse>(`${API_BASE}/sessions/${id}/details`),

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
    getConfigSource: async (): Promise<string> => {
        const token = localStorage.getItem('token');
        const headers: HeadersInit = token ? { 'Authorization': `Bearer ${token}` } : {};

        const response = await fetch(`${API_BASE}/config/source`, { headers });
        if (response.status === 404) return '';
        if (!response.ok) {
            const error = await response.json().catch(() => ({}));
            throw new Error(error.message || `HTTP error! status: ${response.status}`);
        }
        return response.text();
    },
    updateConfigSource: async (content: string): Promise<void> => {
        const token = localStorage.getItem('token');
        const headers: HeadersInit = {
            ...(token ? { 'Authorization': `Bearer ${token}` } : {}),
            // Content-Type not set to let it be raw body? Or text/plain?
            // Axum string extraction usually works with any content type if body is consumed as String
        };

        const response = await fetch(`${API_BASE}/config/source`, {
            method: 'PUT',
            body: content,
            headers
        });
        if (!response.ok) {
            const text = await response.text();
            throw new Error(text || 'Failed to update config source');
        }
    },
    getConfigSchema: () => fetchJson<any>(`${API_BASE}/config/schema`),

    // Scripts API
    executeScript: (req: ExecuteScriptRequest) => fetchJson<ExecuteScriptResponse>(`${API_BASE}/scripts/execute`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify(req)
    }),

    // Tools API
    listTools: () => fetchJson<ToolDefinition[]>(`${API_BASE}/tools`),
    executeTool: (id: string, parameters: Record<string, any>) => fetchJson<ExecuteToolResponse>(`${API_BASE}/tools/${id}/execute`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ parameters })
    }),
};
