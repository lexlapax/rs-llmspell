import { useEffect, useRef } from 'react';
import Convert from 'ansi-to-html';
import clsx from 'clsx';

export interface LogEntry {
    type: 'stdout' | 'stderr' | 'info';
    content: string | object;
    timestamp: number;
    isResult?: boolean;
}

interface ConsoleProps {
    logs: LogEntry[];
    height?: string;
    onClear?: () => void;
    className?: string;
}

const converter = new Convert({
    newline: true,
    escapeXML: true,
    colors: {
        0: '#000000', 1: '#AA0000', 2: '#00AA00', 3: '#AA5500', 4: '#0000AA', 5: '#AA00AA', 6: '#00AAAA', 7: '#AAAAAA',
        8: '#555555', 9: '#FF5555', 10: '#55FF55', 11: '#FFFF55', 12: '#5555FF', 13: '#FF55FF', 14: '#55FFFF', 15: '#FFFFFF'
    }
});

export default function Console({ logs, height = '300px', onClear, className }: ConsoleProps) {
    const bottomRef = useRef<HTMLDivElement>(null);
    const scrollRef = useRef<HTMLDivElement>(null);

    // Auto-scroll to bottom when logs change
    useEffect(() => {
        if (scrollRef.current) {
            const { scrollHeight, clientHeight, scrollTop } = scrollRef.current;
            // Only auto-scroll if we are close to the bottom, or if it's the first load
            const isNearBottom = scrollHeight - scrollTop - clientHeight < 100;

            if (isNearBottom) {
                bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
            }
        }
    }, [logs]);

    // Format timestamp
    const formatTime = (ts: number) => {
        return new Date(ts).toLocaleTimeString([], { hour12: false, hour: '2-digit', minute: '2-digit', second: '2-digit', fractionalSecondDigits: 3 });
    };

    return (
        <div
            className={clsx("flex flex-col bg-[#1e1e1e] border border-gray-700 rounded-lg overflow-hidden", className)}
            style={{ height }}
        >
            <div className="flex items-center justify-between px-4 py-2 bg-[#252526] border-b border-gray-700">
                <h3 className="text-sm font-medium text-gray-300">Console</h3>
                {onClear && (
                    <button
                        onClick={onClear}
                        className="text-xs text-gray-400 hover:text-white px-2 py-1 hover:bg-gray-700 rounded"
                    >
                        Clear
                    </button>
                )}
            </div>

            <div
                ref={scrollRef}
                className="flex-1 overflow-auto p-4 font-mono text-sm leading-6"
                style={{ fontFamily: "'Menlo', 'Monaco', 'Courier New', monospace" }}
            >
                {logs.length === 0 ? (
                    <div className="text-gray-500 italic">No output</div>
                ) : (
                    logs.map((log, index) => {
                        const isObject = typeof log.content === 'object' && log.content !== null;

                        return (
                            <div key={index} className="flex group hover:bg-[#2a2a2a] items-start">
                                <span className="text-gray-600 mr-3 select-none flex-shrink-0 w-24 text-right pt-0.5">
                                    [{formatTime(log.timestamp)}]
                                </span>
                                <div className={clsx("flex-1 whitespace-pre-wrap break-all", {
                                    'text-gray-300': log.type === 'stdout' && !log.isResult,
                                    'text-red-400': log.type === 'stderr',
                                    'text-blue-400': log.type === 'info',
                                    'text-green-400': log.isResult, // Distinct color for results
                                })}>
                                    {isObject ? (
                                        <div className="bg-[#111] p-2 rounded border border-gray-800 mt-1">
                                            <div className="text-xs text-gray-500 mb-1">Result:</div>
                                            <pre className="text-green-300">{JSON.stringify(log.content, null, 2)}</pre>
                                        </div>
                                    ) : (
                                        <span dangerouslySetInnerHTML={{ __html: converter.toHtml(log.content as string) }} />
                                    )}
                                </div>
                            </div>
                        );
                    })
                )}
                <div ref={bottomRef} />
            </div>
        </div>
    );
}
