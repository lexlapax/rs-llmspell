
import { useState, useEffect } from 'react';
import CodeEditor from '../components/editor/CodeEditor';
import Console, { type LogEntry } from '../components/editor/Console';
import { useWebSocket } from '../hooks/useWebSocket';
import clsx from 'clsx';
import { api } from '../api/client';
import type { ToolDefinition } from '../api/types';
import { Play, Terminal, Wrench, Menu } from 'lucide-react';

export const Tools = () => {
    const [activeTab, setActiveTab] = useState<'scripts' | 'tools'>('scripts');

    // Scripts State
    const [code, setCode] = useState('-- Write your script here\nprint("Hello World")');
    const [language, setLanguage] = useState<'javascript' | 'lua'>('lua');
    const [scriptLogs, setScriptLogs] = useState<LogEntry[]>([]);

    // Tools State
    const [tools, setTools] = useState<ToolDefinition[]>([]);
    const [selectedTool, setSelectedTool] = useState<ToolDefinition | null>(null);
    const [toolParams, setToolParams] = useState<Record<string, any>>({});
    const [toolOutput, setToolOutput] = useState<string>('');
    const [toolLoading, setToolLoading] = useState(false);

    // WebSocket for streaming logs (mostly for scripts)
    const { isConnected, connectionState, lastMessage } = useWebSocket('/ws/stream');

    useEffect(() => {
        if (lastMessage && activeTab === 'scripts') {
            // Debug: log ALL messages
            console.log('[WS Event]', lastMessage.event_type, lastMessage);

            // Handle streaming output from kernel
            if (lastMessage.event_type === 'kernel.iopub.stream') {
                const content = lastMessage.data?.content;
                if (content && content.text) {
                    setScriptLogs(prev => [...prev, {
                        type: content.name === 'stderr' ? 'stderr' : 'stdout',
                        content: content.text,
                        timestamp: Date.now()
                    }]);
                    return; // Handled
                }
            }

            // Also handle error output
            if (lastMessage.event_type === 'kernel.iopub.error') {
                const content = lastMessage.data?.content;
                if (content) {
                    const errorText = content.evalue || content.ename || 'Unknown error';
                    setScriptLogs(prev => [...prev, {
                        type: 'stderr',
                        content: errorText,
                        timestamp: Date.now()
                    }]);
                    return;
                }
            }

            // Handle rich output (e.g. JSON results)
            if (lastMessage.event_type === 'kernel.iopub.display_data') {
                const content = lastMessage.data?.content;
                if (content && content.data) {
                    // prioritize json over text
                    const richData = content.data['application/json'] || content.data['text/plain'];

                    if (richData) {
                        setScriptLogs(prev => [...prev, {
                            type: 'stdout', // reuse stdout style but with object content
                            content: richData, // Console.tsx will be updated to handle object
                            timestamp: Date.now(),
                            isResult: true // Optional flag if we want distinct styling later
                        } as LogEntry]);
                        return;
                    }
                }
            }

            // Ignore specific events to avoid noise
            if (lastMessage.event_type === 'kernel.execute_reply' ||
                lastMessage.event_type === 'kernel.execute_request' ||
                lastMessage.event_type === 'kernel.status' ||
                lastMessage.event_type === 'kernel.status_change' ||
                lastMessage.event_type === 'kernel.iopub.status' ||
                lastMessage.event_type === 'kernel.input_request') {
                return;
            }

            // Fallback: log raw message (for debugging unexpected events)
            console.log('[Tools.tsx] Unhandled event type:', lastMessage.event_type, lastMessage);
            // Don't show raw JSON in console UI - too noisy
            // setScriptLogs(prev => [...prev, {
            //     type: 'info',
            //     content: JSON.stringify(lastMessage.data),
            //     timestamp: Date.now()
            // } as LogEntry]);
        }
    }, [lastMessage, activeTab]);

    // Fetch tools mainly when switching to tools tab or initial load if needed
    useEffect(() => {
        if (activeTab === 'tools' && tools.length === 0) {
            api.listTools().then(setTools).catch(err => console.error("Failed to load tools", err));
        }
    }, [activeTab]);

    const handleRunScript = async () => {
        const now = Date.now();
        setScriptLogs(prev => [...prev, { type: 'info', content: `Running ${language} script...`, timestamp: now }]);

        try {
            await api.executeScript({ code, engine: language });
            // Output is streamed via WebSocket, so we don't need to append response.output
            setScriptLogs(prev => [...prev, { type: 'info', content: 'Execution finished.', timestamp: Date.now() }]);
        } catch (err: any) {
            setScriptLogs(prev => [...prev, {
                type: 'stderr',
                content: `Error: ${err.message}`,
                timestamp: Date.now()
            }]);
        }
    };

    const handleExecuteTool = async () => {
        if (!selectedTool) return;
        setToolLoading(true);
        setToolOutput('');

        try {
            // Convert params based on schema types (simple number conversion)
            const params = { ...toolParams };
            const schema = selectedTool.schema;
            if (schema && schema.properties) {
                Object.keys(schema.properties).forEach(key => {
                    const prop = schema.properties[key];
                    if (prop.type === 'integer' || prop.type === 'number') {
                        if (params[key]) params[key] = Number(params[key]);
                    }
                    if (prop.type === 'boolean') {
                        params[key] = params[key] === 'true' || params[key] === true;
                    }
                });
            }

            const response = await api.executeTool(selectedTool.name, params);
            setToolOutput(response.output);
        } catch (err: any) {
            setToolOutput(`Error: ${err.message}`);
        } finally {
            setToolLoading(false);
        }
    };

    // Helper to render dynamic form inputs
    const renderToolForm = () => {
        if (!selectedTool || !selectedTool.schema || !selectedTool.schema.properties) return <div className="text-gray-500 italic">No parameters required</div>;

        return (
            <div className="space-y-4">
                {Object.entries(selectedTool.schema.properties).map(([key, prop]: [string, any]) => (
                    <div key={key}>
                        <label className="block text-sm font-medium text-gray-700">
                            {key} {selectedTool.schema.required?.includes(key) && <span className="text-red-500">*</span>}
                        </label>
                        <p className="text-xs text-gray-500 mb-1">{prop.description}</p>
                        {prop.type === 'boolean' ? (
                            <select
                                className="mt-1 block w-full pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm rounded-md"
                                value={toolParams[key]?.toString() || ''}
                                onChange={e => setToolParams({ ...toolParams, [key]: e.target.value === 'true' })}
                            >
                                <option value="">Select...</option>
                                <option value="true">True</option>
                                <option value="false">False</option>
                            </select>
                        ) : prop.type === 'integer' || prop.type === 'number' ? (
                            <input
                                type="number"
                                className="shadow-sm focus:ring-blue-500 focus:border-blue-500 block w-full sm:text-sm border-gray-300 rounded-md"
                                value={toolParams[key] || ''}
                                onChange={e => setToolParams({ ...toolParams, [key]: e.target.value })}
                            />
                        ) : (
                            <input
                                type="text"
                                className="shadow-sm focus:ring-blue-500 focus:border-blue-500 block w-full sm:text-sm border-gray-300 rounded-md"
                                value={toolParams[key] || ''}
                                onChange={e => setToolParams({ ...toolParams, [key]: e.target.value })}
                            />
                        )}
                    </div>
                ))}
            </div>
        );
    };

    return (
        <div className="p-6 space-y-6 flex flex-col h-[calc(100vh-4rem)]">
            <div className="flex items-center justify-between shrink-0">
                <div className="flex items-center space-x-4">
                    <h1 className="text-2xl font-bold text-gray-900">Execution Environment</h1>
                    <span className={clsx(
                        "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium",
                        isConnected ? "bg-green-100 text-green-800" : "bg-red-100 text-red-800"
                    )}>
                        WS: {connectionState}
                    </span>
                </div>

                {/* Tabs */}
                <div className="flex space-x-2 bg-gray-100 p-1 rounded-lg">
                    <button
                        onClick={() => setActiveTab('scripts')}
                        className={clsx(
                            "px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center space-x-2",
                            activeTab === 'scripts' ? "bg-white shadow text-blue-700" : "text-gray-600 hover:text-gray-900"
                        )}
                    >
                        <Terminal className="w-4 h-4" />
                        <span>Scripts</span>
                    </button>
                    <button
                        onClick={() => setActiveTab('tools')}
                        className={clsx(
                            "px-4 py-2 rounded-md text-sm font-medium transition-colors flex items-center space-x-2",
                            activeTab === 'tools' ? "bg-white shadow text-blue-700" : "text-gray-600 hover:text-gray-900"
                        )}
                    >
                        <Wrench className="w-4 h-4" />
                        <span>Tools</span>
                    </button>
                </div>
            </div>

            {/* TAB CONTENT */}
            {activeTab === 'scripts' ? (
                <div className="bg-white shadow rounded-lg overflow-hidden flex flex-col flex-1 min-h-0">
                    <div className="px-6 py-4 border-b border-gray-200 shrink-0 flex justify-between items-center bg-gray-50">
                        <div className="flex items-center space-x-3">
                            <span className="text-sm font-medium text-gray-700">Language:</span>
                            <select
                                value={language}
                                onChange={(e) => setLanguage(e.target.value as 'javascript' | 'lua')}
                                className="block w-32 pl-3 pr-10 py-1.5 text-base border-gray-300 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm rounded-md"
                            >
                                <option value="lua">Lua</option>
                                <option value="javascript">JavaScript</option>
                            </select>
                        </div>
                        <button
                            className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                            onClick={handleRunScript}
                        >
                            <Play className="w-4 h-4 mr-2" />
                            Run Script
                        </button>
                    </div>

                    <div className="flex-1 flex flex-col min-h-0">
                        <div className="flex-1 min-h-0 border-b border-gray-200 relative">
                            <CodeEditor
                                value={code}
                                onChange={(val: string | undefined) => setCode(val || '')}
                                language={language}
                                height="100%"
                            />
                        </div>
                        <div className="h-64 shrink-0 bg-[#1e1e1e]">
                            <Console
                                logs={scriptLogs}
                                onClear={() => setScriptLogs([])}
                                height="100%"
                                className="rounded-none border-x-0 border-b-0"
                            />
                        </div>
                    </div>
                </div>
            ) : (
                <div className="flex flex-1 min-h-0 gap-6">
                    {/* Tool Selection Sidebar */}
                    <div className="w-64 bg-white shadow rounded-lg flex flex-col overflow-hidden">
                        <div className="p-4 border-b border-gray-200 bg-gray-50 font-medium text-gray-700 flex items-center">
                            <Menu className="w-4 h-4 mr-2" /> Available Tools
                        </div>
                        <div className="overflow-y-auto flex-1 p-2 space-y-1">
                            {tools.map(tool => (
                                <button
                                    key={tool.name}
                                    onClick={() => { setSelectedTool(tool); setToolParams({}); setToolOutput(''); }}
                                    className={clsx(
                                        "w-full text-left px-3 py-2 rounded-md text-sm transition-colors",
                                        selectedTool?.name === tool.name
                                            ? "bg-blue-50 text-blue-700 font-medium"
                                            : "text-gray-600 hover:bg-gray-50"
                                    )}
                                >
                                    <div className="font-medium">{tool.name}</div>
                                    <div className="text-xs text-gray-500 truncate">{tool.category}</div>
                                </button>
                            ))}
                            {tools.length === 0 && <div className="p-4 text-center text-sm text-gray-500">No tools found</div>}
                        </div>
                    </div>

                    {/* Tool Execution Area */}
                    <div className="flex-1 bg-white shadow rounded-lg flex flex-col overflow-hidden">
                        {selectedTool ? (
                            <>
                                <div className="px-6 py-4 border-b border-gray-200 bg-gray-50">
                                    <h2 className="text-lg font-bold text-gray-900">{selectedTool.name}</h2>
                                    <p className="text-sm text-gray-500">{selectedTool.description}</p>
                                </div>
                                <div className="flex-1 overflow-y-auto p-6">
                                    <div className="max-w-xl space-y-6">
                                        <div className="bg-gray-50 p-4 rounded-md border border-gray-200">
                                            <h3 className="text-sm font-medium text-gray-900 mb-4 border-b border-gray-200 pb-2">Parameters</h3>
                                            {renderToolForm()}
                                        </div>
                                        <button
                                            onClick={handleExecuteTool}
                                            disabled={toolLoading}
                                            className="w-full inline-flex justify-center items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
                                        >
                                            {toolLoading ? 'Executing...' : 'Execute Tool'}
                                        </button>

                                        {toolOutput && (
                                            <div className="mt-8">
                                                <h3 className="text-sm font-medium text-gray-900 mb-2">Result</h3>
                                                <pre className="bg-[#1e1e1e] text-gray-200 p-4 rounded-md overflow-x-auto text-sm font-mono whitespace-pre-wrap">
                                                    {toolOutput}
                                                </pre>
                                            </div>
                                        )}
                                    </div>
                                </div>
                            </>
                        ) : (
                            <div className="flex-1 flex flex-col items-center justify-center text-gray-500">
                                <Wrench className="w-12 h-12 mb-4 opacity-20" />
                                <p>Select a tool from the sidebar to configure and execute it.</p>
                            </div>
                        )}
                    </div>
                </div>
            )}
        </div>
    );
};
