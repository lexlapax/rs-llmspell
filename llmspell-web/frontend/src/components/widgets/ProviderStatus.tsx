import { useState, useEffect } from 'react';
import { api } from '../../api/client';
import type { ConfigItem } from '../../api/types';
import { Server, Activity, Cpu } from 'lucide-react';
import clsx from 'clsx';

export default function ProviderStatus() {
    const [configItems, setConfigItems] = useState<ConfigItem[]>([]);
    const [status, setStatus] = useState<'online' | 'offline' | 'checking'>('checking');
    const [latency, setLatency] = useState<number | null>(null);

    useEffect(() => {
        const fetchConfig = async () => {
            try {
                const cfg = await api.getConfig();
                setConfigItems(cfg);
            } catch (err) {
                console.error("Failed to fetch config", err);
            }
        };
        fetchConfig();
    }, []);

    // Simulate status check polling
    useEffect(() => {
        const checkStatus = () => {
            // To be replaced with real /api/status endpoint
            const start = Date.now();
            setTimeout(() => {
                const ms = Date.now() - start;
                setLatency(ms); // Simulate ~good latency
                setStatus('online');
            }, Math.random() * 200 + 50); // 50-250ms simulated response
        };

        checkStatus();
        const interval = setInterval(checkStatus, 30000); // Check every 30s
        return () => clearInterval(interval);
    }, []);

    const getProviderIcon = (provider?: string) => {
        switch (provider?.toLowerCase()) {
            case 'ollama': return <Cpu className="w-5 h-5 text-orange-500" />;
            case 'openai': return <Activity className="w-5 h-5 text-green-500" />;
            case 'candle': return <Server className="w-5 h-5 text-blue-500" />;
            default: return <Server className="w-5 h-5 text-gray-500" />;
        }
    };

    // Fallback if config isn't loaded yet
    const activeProviderItem = configItems.find(c => c.name === 'active_provider') || configItems.find(c => c.name === 'llm.provider');
    const providerName = activeProviderItem?.value || 'Simulated';

    return (
        <div className="bg-white rounded-lg shadow p-4 flex items-center justify-between border border-gray-100">
            <div className="flex items-center space-x-3">
                <div className="p-2 bg-gray-50 rounded-lg">
                    {getProviderIcon(providerName)}
                </div>
                <div>
                    <h3 className="text-sm font-medium text-gray-700">Active Provider</h3>
                    <p className="text-lg font-bold text-gray-900 capitalize flex items-center gap-2">
                        {providerName}
                        {status === 'online' && <span className="flex h-2 w-2 relative">
                            <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                            <span className="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span>
                        </span>}
                    </p>
                </div>
            </div>

            <div className="text-right">
                <div className={clsx(
                    "text-xs font-medium px-2 py-1 rounded-full inline-block",
                    status === 'online' ? "bg-green-50 text-green-700" :
                        status === 'offline' ? "bg-red-50 text-red-700" : "bg-gray-50 text-gray-600"
                )}>
                    {status === 'online' ? 'Operational' : status === 'offline' ? 'Offline' : 'Checking...'}
                </div>
                {latency && (
                    <p className="text-xs text-xs text-gray-400 mt-1 font-mono">
                        ~{latency}ms
                    </p>
                )}
            </div>
        </div>
    );
}
