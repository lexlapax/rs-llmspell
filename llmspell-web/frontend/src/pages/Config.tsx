
import { useEffect, useState, useCallback } from 'react';
import { api } from '../api/client';
import type { ConfigItem } from '../api/types';
import { ConfigTable } from '../components/config/ConfigTable';
import { Settings, RefreshCw, AlertTriangle, Save, X, FileJson, Code, List } from 'lucide-react';
import Editor from '@monaco-editor/react';
import Form from '@rjsf/core';
import validator from '@rjsf/validator-ajv8';
import { parse, stringify } from 'smol-toml';

export const Config = () => {
    const [activeTab, setActiveTab] = useState<'runtime' | 'files'>('runtime');

    // Runtime Config State
    const [config, setConfig] = useState<ConfigItem[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [editingItem, setEditingItem] = useState<ConfigItem | null>(null);
    const [editValue, setEditValue] = useState('');
    const [updating, setUpdating] = useState(false);

    // Static Config State
    const [fileContent, setFileContent] = useState('');
    const [schema, setSchema] = useState<any>(null);
    const [formData, setFormData] = useState<any>(null);
    const [fileLoading, setFileLoading] = useState(false);
    const [fileSaving, setFileSaving] = useState(false);
    const [viewMode, setViewMode] = useState<'source' | 'form'>('source');
    const [parseError, setParseError] = useState<string | null>(null);

    const loadRuntimeConfig = useCallback(async () => {
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
    }, []);

    const loadStaticConfig = useCallback(async () => {
        setFileLoading(true);
        setParseError(null);
        try {
            const [source, schemaData] = await Promise.all([
                api.getConfigSource(),
                api.getConfigSchema()
            ]);
            setFileContent(source);
            setSchema(schemaData);

            // Try parsing generic TOML to JSON for form
            try {
                if (source) {
                    const parsed = parse(source);
                    setFormData(parsed);
                }
            } catch (e) {
                console.warn("Failed to parse initial TOML", e);
                // Don't error blocking UI, just fallback to source view
            }
        } catch (err) {
            setParseError(err instanceof Error ? err.message : 'Failed to load static configuration');
        } finally {
            setFileLoading(false);
        }
    }, []);

    useEffect(() => {
        if (activeTab === 'runtime') {
            loadRuntimeConfig();
        } else {
            loadStaticConfig();
        }
    }, [activeTab, loadRuntimeConfig, loadStaticConfig]);

    // Runtime Handlers
    const handleEdit = (item: ConfigItem) => {
        setEditingItem(item);
        setEditValue(item.is_sensitive ? '' : item.value || '');
    };

    const handleSaveRuntime = async () => {
        if (!editingItem) return;
        setUpdating(true);
        try {
            const overrides = { [editingItem.name]: editValue };
            await api.updateConfig(overrides);
            await loadRuntimeConfig();
            setEditingItem(null);
        } catch (err) {
            alert('Failed to update config: ' + (err instanceof Error ? err.message : 'Unknown error'));
        } finally {
            setUpdating(false);
        }
    };

    // Static Config Handlers
    const handleSaveFile = async () => {
        setFileSaving(true);
        try {
            let contentToSave = fileContent;

            // If in form mode, convert form data back to TOML
            if (viewMode === 'form' && formData) {
                try {
                    contentToSave = stringify(formData);
                    setFileContent(contentToSave);
                } catch (e) {
                    throw new Error("Failed to convert Form Data to TOML: " + e);
                }
            }

            await api.updateConfigSource(contentToSave);
            alert("Configuration saved. You must restart the server/kernel for changes to take effect.");
            await loadStaticConfig(); // Reload to ensure sync
        } catch (err) {
            alert('Failed to save file: ' + (err instanceof Error ? err.message : 'Unknown error'));
        } finally {
            setFileSaving(false);
        }
    };

    const handleFormChange = (e: any) => {
        setFormData(e.formData);
        // Optional: update source real-time? No, messy.
    };

    const handleEditorChange = (value: string | undefined) => {
        if (value !== undefined) {
            setFileContent(value);
            // Clear parse error if it parses now
            try {
                const parsed = parse(value);
                setFormData(parsed);
                setParseError(null);
            } catch (e: any) {
                // Squelch or show tiny error
                // Don't block editing
            }
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

                <div className="flex gap-2">
                    <button
                        onClick={() => setActiveTab('runtime')}
                        className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${activeTab === 'runtime'
                                ? 'bg-purple-600 text-white'
                                : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
                            }`}
                    >
                        <span className="flex items-center gap-2">
                            <List className="w-4 h-4" />
                            Runtime
                        </span>
                    </button>
                    <button
                        onClick={() => setActiveTab('files')}
                        className={`px-4 py-2 rounded-lg text-sm font-medium transition-colors ${activeTab === 'files'
                                ? 'bg-purple-600 text-white'
                                : 'bg-gray-800 text-gray-400 hover:bg-gray-700'
                            }`}
                    >
                        <span className="flex items-center gap-2">
                            <FileJson className="w-4 h-4" />
                            Files (Static)
                        </span>
                    </button>
                    <button
                        onClick={activeTab === 'runtime' ? loadRuntimeConfig : loadStaticConfig}
                        className="p-2 ml-2 hover:bg-gray-800 rounded-lg text-gray-400 hover:text-white transition-colors"
                    >
                        <RefreshCw className={`w-4 h-4 ${loading || fileLoading ? 'animate-spin' : ''}`} />
                    </button>
                </div>
            </div>

            {/* Tab Content */}
            {activeTab === 'runtime' ? (
                <>
                    {/* Persistent Config Banner */}
                    <div className="mb-6 bg-blue-900/20 border border-blue-800/50 rounded-lg p-3 flex items-start gap-3">
                        <Settings className="w-5 h-5 text-blue-500 flex-shrink-0 mt-0.5" />
                        <div className="text-sm text-blue-200/80">
                            <p className="font-medium text-blue-400">Live Configuration</p>
                            <p>Changes are persisted to the system database (SQLite) and apply immediately to the running kernel.</p>
                        </div>
                    </div>

                    <div className="flex-1 overflow-auto bg-gray-900/30 rounded-xl border border-gray-800 p-4">
                        {error ? (
                            <div className="text-center py-12 text-red-400">
                                <p>{error}</p>
                                <button onClick={loadRuntimeConfig} className="mt-4 text-sm underline">Retry</button>
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
                </>
            ) : (
                <>
                    <div className="flex-1 flex flex-col bg-gray-900/30 rounded-xl border border-gray-800 overflow-hidden">
                        {/* Toolbar */}
                        <div className="p-3 border-b border-gray-800 flex items-center justify-between bg-gray-900/50">
                            <div className="flex items-center gap-2 text-sm text-gray-400">
                                <span className="font-mono text-xs bg-gray-800 px-2 py-1 rounded">llmspell.toml</span>
                                {parseError && <span className="text-red-400 text-xs flex items-center gap-1"><AlertTriangle className="w-3 h-3" /> Parse Error</span>}
                            </div>
                            <div className="flex gap-2">
                                <div className="bg-gray-800 rounded-lg p-1 flex mr-2">
                                    <button
                                        onClick={() => setViewMode('source')}
                                        className={`px-3 py-1 text-xs rounded-md transition-colors ${viewMode === 'source' ? 'bg-gray-700 text-white' : 'text-gray-400 hover:text-white'}`}
                                    >
                                        <span className="flex items-center gap-1"><Code className="w-3 h-3" /> Source</span>
                                    </button>
                                    <button
                                        onClick={() => setViewMode('form')}
                                        className={`px-3 py-1 text-xs rounded-md transition-colors ${viewMode === 'form' ? 'bg-gray-700 text-white' : 'text-gray-400 hover:text-white'}`}
                                    >
                                        <span className="flex items-center gap-1"><List className="w-3 h-3" /> Form</span>
                                    </button>
                                </div>
                                <button
                                    onClick={handleSaveFile}
                                    disabled={fileSaving}
                                    className="px-3 py-1.5 bg-purple-600 hover:bg-purple-500 text-white rounded text-xs font-medium flex items-center gap-2 disabled:opacity-50 transition-colors"
                                >
                                    {fileSaving ? <RefreshCw className="w-3 h-3 animate-spin" /> : <Save className="w-3 h-3" />}
                                    Save & Apply
                                </button>
                            </div>
                        </div>

                        {/* Editor/Form Area */}
                        <div className="flex-1 overflow-hidden relative">
                            {fileLoading ? (
                                <div className="absolute inset-0 flex items-center justify-center">
                                    <RefreshCw className="w-8 h-8 text-purple-500 animate-spin" />
                                </div>
                            ) : viewMode === 'source' ? (
                                <Editor
                                    height="100%"
                                    defaultLanguage="toml" // Monoco might assume plain text if toml not registered, but usually works ok or fallsback
                                    value={fileContent}
                                    theme="vs-dark"
                                    onChange={handleEditorChange}
                                    options={{
                                        minimap: { enabled: false },
                                        fontSize: 14,
                                        scrollBeyondLastLine: false,
                                        wordWrap: 'on'
                                    }}
                                />
                            ) : (
                                <div className="h-full overflow-auto p-6 scrollbar-thin">
                                    <div className="max-w-4xl mx-auto bg-gray-900 p-6 rounded-lg border border-gray-700">
                                        {schema ? (
                                            <Form
                                                schema={schema}
                                                validator={validator}
                                                formData={formData}
                                                onChange={handleFormChange}
                                                uiSchema={{
                                                    "ui:submitButtonOptions": { norender: true } // Hide default submit
                                                }}
                                                className="rjsf-dark-theme" // We might need to style this manually
                                            />
                                        ) : (
                                            <div className="text-center text-gray-500">Schema not available</div>
                                        )}
                                    </div>
                                </div>
                            )}
                        </div>
                    </div>
                </>
            )}

            {/* Edit Modal (Runtime) */}
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
                                onClick={handleSaveRuntime}
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
