export interface WebSocketEvent {
    type: string;
    payload: unknown;
    timestamp: number;
}

export interface ConfigItem {
    name: string;
    description: string;
    category: string;
    value: string | null;
    default: string | null;
    is_sensitive: boolean;
    is_overridden: boolean;
}

export interface UpdateConfigResponse {
    status: string;
    message: string;
    overrides: Record<string, string>;
}

export type ConnectionState = 'CONNECTING' | 'OPEN' | 'CLOSED' | 'ERROR';

export interface UseWebSocketReturn {
    isConnected: boolean;
    connectionState: ConnectionState;
    lastMessage: WebSocketEvent | null;
    send: (msg: unknown) => void;
}
