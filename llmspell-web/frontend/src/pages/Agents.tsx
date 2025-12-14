import { useState } from 'react';
import {
    Bot,
    Square,
    RotateCw,
    Clock,
    MessageSquare,
    Database,
    Code,
    Search,
    Plus
} from 'lucide-react';
import clsx from 'clsx';
import type { AgentInstance, AgentCatalogItem } from '../api/types';

// Mock Data
const MOCK_INSTANCES: AgentInstance[] = [
    {
        id: 'inst_alpha', type_id: 'assistant_v1', name: 'Support Bot Alpha',
        status: 'active', session_id: 'sess_123',
        uptime_seconds: 3600, messages_processed: 142, last_active: '2023-12-05T10:00:00Z'
    },
    {
        id: 'inst_beta', type_id: 'researcher_v1', name: 'Deep Research Unit',
        status: 'working', session_id: 'sess_456',
        uptime_seconds: 1200, messages_processed: 45, last_active: '2023-12-05T10:30:00Z'
    } as any, // 'working' mapped to active visually or need update type
    {
        id: 'inst_gamma', type_id: 'coder_v1', name: 'Code Generator',
        status: 'sleeping', session_id: 'sess_789',
        uptime_seconds: 7200, messages_processed: 0, last_active: '2023-12-05T09:00:00Z'
    },
    {
        id: 'inst_delta', type_id: 'data_v1', name: 'Data ETL Worker',
        status: 'terminated', session_id: 'sess_000',
        uptime_seconds: 500, messages_processed: 12, last_active: '2023-12-04T15:00:00Z'
    },
];

const MOCK_CATALOG: AgentCatalogItem[] = [
    {
        id: 'assistant_v1', name: 'General Assistant', description: 'Standard LLM-backed conversational agent.',
        version: '1.0.0', category: 'assistant', capabilities: ['chat', 'memory'], icon: 'bot'
    },
    {
        id: 'researcher_v1', name: 'Deep Researcher', description: 'Autonomous research agent with web search capabilities.',
        version: '0.9.5', category: 'researcher', capabilities: ['search', 'summarize', 'citation'], icon: 'search'
    },
    {
        id: 'coder_v1', name: 'Rust Architect', description: 'Specialized code generation agent for Rust projects.',
        version: '1.2.0', category: 'coder', capabilities: ['code_gen', 'refactor', 'test'], icon: 'code'
    },
    {
        id: 'data_v1', name: 'Data Analyst', description: 'Structured data processing and visualization expert.',
        version: '0.8.0', category: 'data', capabilities: ['sql', 'pandas', 'chart'], icon: 'database'
    },
];

export default function Agents() {
    const [activeTab, setActiveTab] = useState<'instances' | 'catalog'>('instances');
    const [instances, setInstances] = useState<AgentInstance[]>(MOCK_INSTANCES);

    const getStatusColor = (status: string) => {
        switch (status) {
            case 'active': return 'text-green-600 bg-green-50 ring-green-500/10';
            case 'working': return 'text-blue-600 bg-blue-50 ring-blue-500/10';
            case 'sleeping': return 'text-yellow-600 bg-yellow-50 ring-yellow-500/10';
            case 'terminated': return 'text-gray-600 bg-gray-50 ring-gray-500/10';
            case 'failed': return 'text-red-600 bg-red-50 ring-red-500/10';
            default: return 'text-gray-600 bg-gray-50';
        }
    };

    const getIcon = (iconName?: string) => {
        switch (iconName) {
            case 'search': return <Search className="w-6 h-6" />;
            case 'code': return <Code className="w-6 h-6" />;
            case 'database': return <Database className="w-6 h-6" />;
            default: return <Bot className="w-6 h-6" />;
        }
    };

    const handleAction = (id: string, action: 'stop' | 'restart') => {
        console.log(`${action} agent ${id}`);
        // Mock update
        setInstances(prev => prev.map(inst => {
            if (inst.id === id) {
                if (action === 'stop') return { ...inst, status: 'terminated' as any };
                if (action === 'restart') return { ...inst, status: 'active' };
            }
            return inst;
        }));
    };

    return (
        <div className="p-6 max-w-7xl mx-auto space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-2xl font-bold text-gray-900 flex items-center gap-3">
                        <Bot className="w-8 h-8 text-indigo-600" />
                        Agents
                    </h1>
                    <p className="mt-1 text-sm text-gray-500">
                        Manage active agent instances and explore the agent catalog.
                    </p>
                </div>

                <div className="flex bg-gray-100 p-1 rounded-lg">
                    <button
                        onClick={() => setActiveTab('instances')}
                        className={clsx(
                            "px-4 py-2 text-sm font-medium rounded-md transition-all",
                            activeTab === 'instances'
                                ? "bg-white text-gray-900 shadow-sm"
                                : "text-gray-500 hover:text-gray-700"
                        )}
                    >
                        Active Instances
                        <span className="ml-2 bg-indigo-100 text-indigo-600 py-0.5 px-2 rounded-full text-xs">
                            {instances.filter(i => i.status !== 'terminated').length}
                        </span>
                    </button>
                    <button
                        onClick={() => setActiveTab('catalog')}
                        className={clsx(
                            "px-4 py-2 text-sm font-medium rounded-md transition-all",
                            activeTab === 'catalog'
                                ? "bg-white text-gray-900 shadow-sm"
                                : "text-gray-500 hover:text-gray-700"
                        )}
                    >
                        Catalog
                    </button>
                </div>
            </div>

            {/* Content Area */}
            {activeTab === 'instances' ? (
                <div className="bg-white shadow rounded-lg overflow-hidden border border-gray-200">
                    <table className="min-w-full divide-y divide-gray-200">
                        <thead className="bg-gray-50">
                            <tr>
                                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Agent Instance</th>
                                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Status</th>
                                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Session</th>
                                <th className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">Metrics</th>
                                <th className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">Actions</th>
                            </tr>
                        </thead>
                        <tbody className="bg-white divide-y divide-gray-200">
                            {instances.map((instance) => (
                                <tr key={instance.id} className="hover:bg-gray-50">
                                    <td className="px-6 py-4 whitespace-nowrap">
                                        <div className="flex items-center">
                                            <div className="flex-shrink-0 h-10 w-10 flex items-center justify-center bg-indigo-50 rounded-full text-indigo-600">
                                                <Bot className="w-5 h-5" />
                                            </div>
                                            <div className="ml-4">
                                                <div className="text-sm font-medium text-gray-900">{instance.name}</div>
                                                <div className="text-xs text-gray-500 font-mono">{instance.id}</div>
                                            </div>
                                        </div>
                                    </td>
                                    <td className="px-6 py-4 whitespace-nowrap">
                                        <span className={clsx(
                                            "inline-flex items-center rounded-md px-2 py-1 text-xs font-medium ring-1 ring-inset capitalize",
                                            getStatusColor(instance.status)
                                        )}>
                                            {instance.status === 'active' && <span className="mr-1.5 flex h-2 w-2 relative">
                                                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-green-400 opacity-75"></span>
                                                <span className="relative inline-flex rounded-full h-2 w-2 bg-green-500"></span>
                                            </span>}
                                            {instance.status}
                                        </span>
                                    </td>
                                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                        {instance.session_id ? (
                                            <a href={`/sessions/${instance.session_id}`} className="text-indigo-600 hover:underline">
                                                {instance.session_id}
                                            </a>
                                        ) : (
                                            <span className="text-gray-400">-</span>
                                        )}
                                    </td>
                                    <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500 space-y-1">
                                        <div className="flex items-center gap-2" title="Uptime">
                                            <Clock className="w-4 h-4 text-gray-400" />
                                            <span>{Math.floor(instance.uptime_seconds / 60)}m</span>
                                        </div>
                                        <div className="flex items-center gap-2" title="Messages Processed">
                                            <MessageSquare className="w-4 h-4 text-gray-400" />
                                            <span>{instance.messages_processed} msgs</span>
                                        </div>
                                    </td>
                                    <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                        <div className="flex items-center justify-end gap-2">
                                            {instance.status !== 'terminated' && (
                                                <button
                                                    onClick={() => handleAction(instance.id, 'stop')}
                                                    className="text-red-600 hover:text-red-900 p-1 rounded hover:bg-red-50"
                                                    title="Stop Agent"
                                                >
                                                    <Square className="w-4 h-4 fill-current" />
                                                </button>
                                            )}
                                            {instance.status === 'terminated' && (
                                                <button
                                                    onClick={() => handleAction(instance.id, 'restart')}
                                                    className="text-green-600 hover:text-green-900 p-1 rounded hover:bg-green-50"
                                                    title="Restart Agent"
                                                >
                                                    <RotateCw className="w-4 h-4" />
                                                </button>
                                            )}
                                        </div>
                                    </td>
                                </tr>
                            ))}
                        </tbody>
                    </table>
                </div>
            ) : (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-6">
                    {MOCK_CATALOG.map((item) => (
                        <div key={item.id} className="bg-white rounded-lg shadow border border-gray-200 p-6 hover:shadow-md transition-shadow">
                            <div className="flex items-start justify-between mb-4">
                                <div className="p-3 bg-indigo-50 rounded-lg text-indigo-600">
                                    {getIcon(item.icon)}
                                </div>
                                <span className="inline-flex items-center rounded-md bg-gray-50 px-2 py-1 text-xs font-medium text-gray-600 ring-1 ring-inset ring-gray-500/10">
                                    v{item.version}
                                </span>
                            </div>
                            <h3 className="text-lg font-semibold text-gray-900 mb-1">{item.name}</h3>
                            <p className="text-sm text-gray-500 mb-4 h-10 line-clamp-2">{item.description}</p>

                            <div className="flex flex-wrap gap-2 mb-6">
                                {item.capabilities.map(cap => (
                                    <span key={cap} className="inline-flex items-center rounded-full bg-blue-50 px-2 py-1 text-xs font-medium text-blue-700 ring-1 ring-inset ring-blue-700/10">
                                        {cap}
                                    </span>
                                ))}
                            </div>

                            <button className="w-full flex items-center justify-center gap-2 bg-indigo-600 text-white px-4 py-2 rounded-md hover:bg-indigo-700 transition-colors text-sm font-medium">
                                <Plus className="w-4 h-4" />
                                Deploy Agent
                            </button>
                        </div>
                    ))}
                </div>
            )}
        </div>
    );
}
