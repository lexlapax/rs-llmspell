import { useState, useCallback } from 'react';
import CodeEditor from '../components/editor/CodeEditor';
import Console, { type LogEntry } from '../components/editor/Console';
import { useWebSocket } from '../hooks/useWebSocket';
import clsx from 'clsx';

export default function Tools() {
    const [code, setCode] = useState('-- Write your script here\nprint("Hello World")');
    const [language, setLanguage] = useState<'javascript' | 'lua'>('lua');
    const [logs, setLogs] = useState<LogEntry[]>([]);
    const { isConnected, connectionState } = useWebSocket('/ws/stream');

    const handleRun = useCallback(() => {
        // Simulate execution logs for now
        const now = Date.now();
        setLogs(prev => [
            ...prev,
            { type: 'info', content: `Running ${language} script...`, timestamp: now },
            { type: 'stdout', content: 'Hello World', timestamp: now + 10 },
            { type: 'stdout', content: '\x1b[32mProcess started successfully.\x1b[0m', timestamp: now + 20 },
            { type: 'stderr', content: '\x1b[31mWarning: This is a simulated error/warning for testing ANSI colors.\x1b[0m', timestamp: now + 30 },
            { type: 'info', content: 'Execution finished.', timestamp: now + 40 },
        ]);
    }, [language]);

    return (
        <div className="p-6 space-y-6 flex flex-col h-[calc(100vh-4rem)]">
            <div className="flex items-center justify-between shrink-0">
                <div className="flex items-center space-x-4">
                    <h1 className="text-2xl font-bold text-gray-900">Tools & Scripts</h1>
                    <span className={clsx(
                        "inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium",
                        isConnected ? "bg-green-100 text-green-800" : "bg-red-100 text-red-800"
                    )}>
                        WS: {connectionState}
                    </span>
                </div>
                <div className="flex space-x-2">
                    <select
                        value={language}
                        onChange={(e) => setLanguage(e.target.value as 'javascript' | 'lua')}
                        className="block w-48 pl-3 pr-10 py-2 text-base border-gray-300 focus:outline-none focus:ring-blue-500 focus:border-blue-500 sm:text-sm rounded-md"
                    >
                        <option value="lua">Lua</option>
                        <option value="javascript">JavaScript</option>
                    </select>
                </div>
            </div>

            <div className="bg-white shadow rounded-lg overflow-hidden flex flex-col flex-1 min-h-0">
                <div className="px-6 py-4 border-b border-gray-200 shrink-0 flex justify-between items-center">
                    <h2 className="text-lg font-medium text-gray-900">Script Editor</h2>
                    <button
                        className="inline-flex items-center px-4 py-2 border border-transparent text-sm font-medium rounded-md shadow-sm text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                        onClick={handleRun}
                    >
                        Run Script
                    </button>
                </div>

                <div className="flex-1 flex flex-col min-h-0">
                    <div className="flex-1 min-h-0 border-b border-gray-200 relative">
                        <CodeEditor
                            value={code}
                            onChange={(val) => setCode(val || '')}
                            language={language}
                            height="100%"
                        />
                    </div>
                    <div className="h-64 shrink-0 bg-[#1e1e1e]">
                        <Console
                            logs={logs}
                            onClear={() => setLogs([])}
                            height="100%"
                            className="rounded-none border-x-0 border-b-0"
                        />
                    </div>
                </div>
            </div>
        </div>
    );
}
