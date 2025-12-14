import { useState, useEffect } from 'react';
import { api } from '../api/client';
import type { HealthResponse } from '../api/client';

export function useSystemStatus() {
    const [status, setStatus] = useState<HealthResponse | null>(null);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<Error | null>(null);

    useEffect(() => {
        const fetchStatus = async () => {
            try {
                const data = await api.getHealth();
                setStatus(data);
                setError(null);
            } catch (err) {
                setError(err instanceof Error ? err : new Error('Failed to fetch status'));
            } finally {
                setLoading(false);
            }
        };

        fetchStatus();
        const interval = setInterval(fetchStatus, 30000); // Poll every 30s

        return () => clearInterval(interval);
    }, []);

    return { status, loading, error };
}
