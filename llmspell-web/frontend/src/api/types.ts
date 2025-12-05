export interface WebSocketEvent {
    type: string;
    payload: unknown;
    timestamp: number;
}

export type ConnectionState = 'CONNECTING' | 'OPEN' | 'CLOSED' | 'ERROR';

export interface UseWebSocketReturn {
    isConnected: boolean;
    connectionState: ConnectionState;
    lastMessage: WebSocketEvent | null;
    send: (msg: unknown) => void;
}
