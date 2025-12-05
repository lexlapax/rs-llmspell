import { useState, useEffect, useRef, useCallback } from 'react';
import type { ConnectionState, UseWebSocketReturn, WebSocketEvent } from '../api/types';

const MAX_RECONNECT_ATTEMPTS = 5;

export function useWebSocket(url: string): UseWebSocketReturn {
    const [connectionState, setConnectionState] = useState<ConnectionState>('CLOSED');
    const [lastMessage, setLastMessage] = useState<WebSocketEvent | null>(null);
    const wsRef = useRef<WebSocket | null>(null);
    const reconnectAttempts = useRef(0);
    const reconnectTimer = useRef<number | undefined>(undefined);
    const [connectTrigger, setConnectTrigger] = useState(0);

    useEffect(() => {
        let isMounted = true;
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const fullUrl = url.startsWith('/')
            ? `${protocol}//${window.location.host}${url}`
            : url;

        console.log(`Connecting to ${fullUrl} (Attempt ${reconnectAttempts.current})`);
        // eslint-disable-next-line react-hooks/exhaustive-deps
        setConnectionState('CONNECTING');

        const ws = new WebSocket(fullUrl);
        wsRef.current = ws;

        ws.onopen = () => {
            if (isMounted) {
                setConnectionState('OPEN');
                reconnectAttempts.current = 0;
                console.log('WebSocket connected');
            }
        };

        ws.onclose = () => {
            if (isMounted) {
                setConnectionState('CLOSED');
                wsRef.current = null;
                console.log('WebSocket closed');

                if (reconnectAttempts.current < MAX_RECONNECT_ATTEMPTS) {
                    reconnectAttempts.current += 1;
                    const delay = Math.min(1000 * (2 ** reconnectAttempts.current), 30000);
                    console.log(`Attempting reconnect in ${delay}ms`);

                    reconnectTimer.current = window.setTimeout(() => {
                        if (isMounted) setConnectTrigger(prev => prev + 1);
                    }, delay);
                } else {
                    console.error('Max reconnect attempts reached');
                }
            }
        };

        ws.onerror = (error) => {
            console.error('WebSocket error:', error);
            if (isMounted) setConnectionState('ERROR');
            // ws.close() will be called automatically or manually here
            // but onerror usually precedes onclose.
        };

        ws.onmessage = (event) => {
            if (isMounted) {
                try {
                    const data = JSON.parse(event.data);
                    setLastMessage(data);
                } catch {
                    console.error('Failed to parse WebSocket message:', event.data);
                }
            }
        };

        return () => {
            isMounted = false;
            // Only close if we are unmounting or re-running effect (which creates a new one)
            // But we must check state to avoid errors
            if (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING) {
                ws.close();
            }
            if (reconnectTimer.current) {
                clearTimeout(reconnectTimer.current);
            }
        };
    }, [url, connectTrigger]);

    const send = useCallback((msg: unknown) => {
        if (wsRef.current && wsRef.current.readyState === WebSocket.OPEN) {
            wsRef.current.send(JSON.stringify(msg));
        } else {
            console.error('WebSocket is not open');
        }
    }, []);

    return {
        isConnected: connectionState === 'OPEN',
        connectionState,
        lastMessage,
        send
    };
}
