
import { useEffect, useState } from 'react';
import { api } from '../api/client';
import type { ConfigItem } from '../api/types';
import { ConfigTable } from '../components/config/ConfigTable';
import { Settings, RefreshCw, AlertTriangle, Save, X } from 'lucide-react';

export const Config = () => {
    const [config, setConfig] = useState<ConfigItem[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [editingItem, setEditingItem] = useState<ConfigItem | null>(null);
    const [editValue, setEditValue] = useState('');
    const [updating, setUpdating] = useState(false);

    const loadConfig = async () => {
        setLoading(true);
        setError(null);
        try {
            const data = await api.getConfig();
            setConfig(data);
        } catch (err) {
            setError(err instanceof Error ? err.message : 'Failed to load configuration');
        } finally {
            setLoading(false);
        }
    };

    useEffect(() => {
        loadConfig();
    }, []);

    const handleEdit = (item: ConfigItem) => {
        setEditingItem(item);
        // Clear sensitive values for security (don't pre-fill ***)
        setEditValue(item.is_sensitive ? '' : item.value || '');
    };

    const handleSave = async () => {
        if (!editingItem) return;

        setUpdating(true);
        try {
            const overrides = { [editingItem.name]: editValue };
            await api.updateConfig(overrides);

            // Refresh config to show updates (though backend currently simulates)
            // In simulation mode, the backend doesn't persist, so we might optimistically update UI?
            // Or just show success message.
            // Since backend is simulated, refresh won't show change unless we persisted it in a mock store.
            // But let's follow the standard flow.
            await loadConfig();
            setEditingItem(null);
        } catch (err) {
            alert('Failed to update config: ' + (err instanceof Error ? err.message : 'Unknown error'));
        } finally {
            setUpdating(false);
        }
    };

    return (
        <div className="h-full flex flex-col p-6 overflow-hidden">
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div className="flex items-center gap-3">
                    <div className="p-2 bg-purple-500/10 rounded-lg">
                        <Settings className="w-6 h-6 text-purple-400" />
                    </div>
                    <div>
                        <h1 className="text-xl font-bold text-white">Configuration</h1>
                        <p className="text-xs text-gray-500">Manage environment variables and runtime settings</p>
                    </div>
                </div>
                <button
                    onClick={loadConfig}
                    className="p-2 hover:bg-gray-800 rounded-lg text-gray-400 hover:text-white transition-colors"
                >
                    <RefreshCw className={`w-4 h-4 ${loading ? 'animate-spin' : ''}`} />
                </button>
            </div>

            {/* Persistent Config Banner */}
            <div className="mb-6 bg-blue-900/20 border border-blue-800/50 rounded-lg p-3 flex items-start gap-3">
                <Settings className="w-5 h-5 text-blue-500 flex-shrink-0 mt-0.5" />
                <div className="text-sm text-blue-200/80">
                    <p className="font-medium text-blue-400">Live Configuration</p>
                    <p>Changes are persisted to the system database (SQLite) and apply immediately to the running kernel.</p>
                </div>
            </div>

            {/* Content */}
            <div className="flex-1 overflow-auto bg-gray-900/30 rounded-xl border border-gray-800 p-4">
                {error ? (
                    <div className="text-center py-12 text-red-400">
                        <p>{error}</p>
                        <button onClick={loadConfig} className="mt-4 text-sm underline">Retry</button>
                    </div>
                ) : loading ? (
                    <div className="animate-pulse space-y-4">
                        {[1, 2, 3, 4, 5].map(i => (
                            <div key={i} className="h-12 bg-gray-800/50 rounded-lg w-full"></div>
                        ))}
                    </div>
                ) : (
                    <ConfigTable items={config} onEdit={handleEdit} />
                )}
            </div>

            {/* Edit Modal */}
            {editingItem && (
                <div className="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-center justify-center p-4">
                    <div className="bg-gray-900 border border-gray-700 rounded-xl w-full max-w-lg shadow-2xl">
                        <div className="flex items-center justify-between p-4 border-b border-gray-800">
                            <h3 className="font-medium text-white">Edit Configuration</h3>
                            <button
                                onClick={() => setEditingItem(null)}
                                className="text-gray-400 hover:text-white"
                            >
                                <X className="w-5 h-5" />
                            </button>
                        </div>

                        <div className="p-6 space-y-4">
                            <div>
                                <label className="block text-xs font-mono text-gray-500 mb-1">VARIABLE</label>
                                <div className="font-mono text-blue-400 bg-gray-950 px-3 py-2 rounded border border-gray-800">
                                    {editingItem.name}
                                </div>
                            </div>

                            <div>
                                <label className="block text-xs font-mono text-gray-500 mb-1">VALUE</label>
                                <input
                                    type="text"
                                    value={editValue}
                                    onChange={(e) => setEditValue(e.target.value)}
                                    className="w-full bg-gray-950 border border-gray-700 rounded-lg px-3 py-2 text-white font-mono focus:border-purple-500 focus:outline-none"
                                    placeholder={editingItem.is_sensitive ? "Enter new sensitive value..." : "Enter value..."}
                                />
                                {editingItem.is_sensitive && (
                                    <p className="text-xs text-yellow-500/80 mt-1 flex items-center gap-1">
                                        <AlertTriangle className="w-3 h-3" />
                                        Existing value is hidden for security.
                                    </p>
                                )}
                            </div>

                            <div className="text-xs text-gray-400 bg-gray-800/50 p-3 rounded">
                                {editingItem.description}
                            </div>
                        </div>

                        <div className="flex items-center justify-end gap-3 p-4 border-t border-gray-800 bg-gray-900/50 rounded-b-xl">
                            <button
                                onClick={() => setEditingItem(null)}
                                className="px-4 py-2 text-sm text-gray-400 hover:text-white"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={handleSave}
                                disabled={updating}
                                className="px-4 py-2 bg-purple-600 hover:bg-purple-500 text-white rounded-lg text-sm font-medium flex items-center gap-2 disabled:opacity-50 transition-colors"
                            >
                                {updating ? (
                                    <RefreshCw className="w-4 h-4 animate-spin" />
                                ) : (
                                    <Save className="w-4 h-4" />
                                )}
                                Save Changes
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};
