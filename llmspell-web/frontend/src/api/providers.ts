export interface ProviderCapabilities {
    supports_streaming: boolean;
    supports_multimodal: boolean;
    max_context_tokens?: number;
    max_output_tokens?: number;
    available_models: string[];
    custom_features: Record<string, any>;
}

export interface ProviderInfo {
    name: string;
    capabilities: ProviderCapabilities;
}

export interface ListProvidersResponse {
    status: string;
    providers: ProviderInfo[];
    error?: string;
}

export async function listProviders(): Promise<ProviderInfo[]> {
    const response = await fetch('/api/providers');

    if (!response.ok) {
        throw new Error(`Failed to list providers: ${response.statusText}`);
    }

    const data: ListProvidersResponse = await response.json();

    if (data.status === 'error') {
        throw new Error(data.error || 'Unknown error occurred');
    }

    return data.providers;
}
