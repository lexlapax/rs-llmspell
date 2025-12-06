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
    // ... existing code ...
    send: (msg: unknown) => void;
}

export interface Template {
    id: string;
    name: string;
    description: string;
    category: string | Record<string, string>;
    tags: string[];
    version: string;
    config_schema: Record<string, any>; // Legacy field, might be removed or used for quick access if backend aligns
}

export type ParameterType = 'string' | 'number' | 'integer' | 'boolean' | 'array' | 'object';

export interface ParameterConstraints {
    min?: number;
    max?: number;
    min_length?: number;
    max_length?: number;
    pattern?: string;
    allowed_values?: any[];
}

export interface ParameterSchema {
    name: string;
    description: string;
    type: ParameterType;
    required: boolean;
    default?: any;
    constraints?: ParameterConstraints;
}

export interface ConfigSchema {
    parameters: ParameterSchema[];
    version: string;
}

export interface TemplateDetails {
    metadata: TemplateMetadata;
    schema: ConfigSchema;
}

export interface TemplateMetadata {
    id: string;
    name: string;
    description: string;
    category: string | Record<string, string>;
    tags: string[];
    version: string;
    // author, requires, etc. omit for now if not used
}


export interface LaunchTemplateResponse {
    session_id: string;
    status: string;
    message: string;
}
