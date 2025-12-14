export interface WebSocketEvent {
    event_type: string;
    data: any;
    language?: string;
    metadata?: any;
}

export interface ConfigItem {
    name: string;
    description: string;
    category: string;
    value: string | null;
    default: string | null;
    is_sensitive: boolean;
    is_overridden: boolean;
    config_path?: string;
}

export interface UpdateConfigResponse {
    status: string;
    message: string;
    overrides: Record<string, string>;
}

export interface ConfigSource {
    format: 'toml';
    content: string;
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

export interface WorkflowNode {
    id: string;
    label: string;
    type: string;
    status: 'pending' | 'running' | 'completed' | 'failed' | 'skipped';
    started_at?: string;
    completed_at?: string;
    duration_ms?: number;
    output?: any;
    error?: string;
    position: { x: number; y: number };
}

export interface WorkflowLink {
    source: string;
    target: string;
    label?: string;
}

export interface WorkflowExecution {
    nodes: WorkflowNode[];
    links: WorkflowLink[];
    status: string;
    started_at: string;
    completed_at?: string;
}

export interface ArtifactInfo {
    id: string;
    name: string;
    type: string;
    size: number;
    created_at: string;
}

export interface SessionDetailsResponse {
    id: string;
    name: string;
    template_id?: string;
    template_name?: string;
    created_at: string;
    updated_at: string;
    status: string;
    metadata: Record<string, any>;
    workflow?: WorkflowExecution;
    logs: string[];
    artifacts: ArtifactInfo[];
}

export interface SessionDetails extends SessionDetailsResponse { } // Alias for compatibility if needed


export interface LaunchTemplateResponse {
    session_id: string;
    status: string;
    message: string;
}

export interface RagDocument {
    id: string;
    filename: string;
    type: 'pdf' | 'md' | 'txt' | 'docx';
    size: number;
    status: 'indexed' | 'processing' | 'failed' | 'pending';
    uploaded_at: string;
}

export interface VectorSearchResult {
    id: string;
    content: string;
    score: number; // Similarity score 0-1
    metadata: {
        document_id: string;
        filename: string;
        chunk_index: number;
    };
}

export type AgentStatus = 'active' | 'sleeping' | 'terminated' | 'failed' | 'idle';

export interface AgentInstance {
    id: string;
    type_id: string;
    name: string;
    status: AgentStatus;
    session_id?: string;
    uptime_seconds: number;
    messages_processed: number;
    last_active: string;
}

export interface AgentCatalogItem {
    id: string;
    name: string;
    description: string;
    version: string;
    capabilities: string[];
    category: 'assistant' | 'researcher' | 'coder' | 'data';
    icon?: string;
}

export interface ExecuteScriptRequest {
    code: string;
    engine?: string; // 'lua' | 'javascript'
}

export interface ExecuteScriptResponse {
    output: string;
}

export interface ToolDefinition {
    name: string;
    description: string;
    category: string;
    schema: Record<string, any>; // JSON Schema
}

export interface ExecuteToolRequest {
    parameters: Record<string, any>;
}

export interface ExecuteToolResponse {
    output: string;
}

