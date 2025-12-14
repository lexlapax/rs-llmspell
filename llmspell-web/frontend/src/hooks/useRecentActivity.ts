import { useState, useEffect } from 'react';
import { api } from '../api/client';
import type { Session } from '../api/client';

export function useRecentActivity() {
    const [sessions, setSessions] = useState<Session[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<Error | null>(null);

    useEffect(() => {
        const fetchActivity = async () => {
            try {
                const data = await api.getSessions();
                // Sort by created_at desc and take top 5
                const sorted = data.sort((a, b) => b.created_at - a.created_at).slice(0, 5);
                setSessions(sorted);
                setError(null);
            } catch (err) {
                setError(err instanceof Error ? err : new Error('Failed to fetch activity'));
            } finally {
                setLoading(false);
            }
        };

        fetchActivity();
        const interval = setInterval(fetchActivity, 30000); // Poll every 30s

        return () => clearInterval(interval);
    }, []);

    return { sessions, loading, error };
}
