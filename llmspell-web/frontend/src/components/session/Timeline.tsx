import { useState, useEffect, useRef } from 'react';
import { Play, Pause, Square, ChevronRight, Clock, AlertCircle, Terminal } from 'lucide-react';
import clsx from 'clsx';

export interface TimelineEvent {
    id: string;
    timestamp: number;
    type: 'user' | 'assistant' | 'system' | 'error' | 'tool';
    title: string;
    content: string;
    duration?: number;
}

interface TimelineProps {
    events: TimelineEvent[];
    onEventSelect?: (event: TimelineEvent) => void;
}

export default function Timeline({ events, onEventSelect }: TimelineProps) {
    const [isPlaying, setIsPlaying] = useState(false);
    const [currentIndex, setCurrentIndex] = useState(0);
    const [playbackSpeed, setPlaybackSpeed] = useState(1); // 1x, 2x, etc.
    const scrollRef = useRef<HTMLDivElement>(null);

    // Auto-scroll to current event
    useEffect(() => {
        if (scrollRef.current) {
            const eventEl = scrollRef.current.children[currentIndex] as HTMLElement;
            if (eventEl) {
                eventEl.scrollIntoView({ behavior: 'smooth', block: 'center' });
            }
        }
    }, [currentIndex]);

    // Playback logic
    useEffect(() => {
        let interval: ReturnType<typeof setInterval>;
        if (isPlaying) {
            interval = setInterval(() => {
                setCurrentIndex(prev => {
                    if (prev >= events.length - 1) {
                        setIsPlaying(false);
                        return prev;
                    }
                    return prev + 1;
                });
            }, 1000 / playbackSpeed);
        }
        return () => clearInterval(interval);
    }, [isPlaying, events.length, playbackSpeed]);

    const handleStep = (index: number) => {
        setCurrentIndex(index);
        setIsPlaying(false);
        onEventSelect?.(events[index]);
    };

    const formatTime = (ms: number) => {
        const date = new Date(ms);
        return date.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' });
    };

    const getIcon = (type: TimelineEvent['type']) => {
        switch (type) {
            case 'user': return <ChevronRight className="w-4 h-4 text-blue-500" />;
            case 'assistant': return <Terminal className="w-4 h-4 text-purple-500" />;
            case 'system': return <Clock className="w-4 h-4 text-gray-500" />;
            case 'error': return <AlertCircle className="w-4 h-4 text-red-500" />;
            case 'tool': return <Square className="w-4 h-4 text-orange-500" />;
        }
    };

    const currentEvent = events[currentIndex];

    return (
        <div className="flex flex-col h-full bg-white border border-gray-200 rounded-lg shadow-sm overflow-hidden">
            {/* Header / Controls */}
            <div className="p-4 border-b border-gray-200 bg-gray-50 flex items-center justify-between shrink-0">
                <div className="flex items-center space-x-2">
                    <button
                        onClick={() => setIsPlaying(!isPlaying)}
                        className={clsx(
                            "p-2 rounded-full transition-colors",
                            isPlaying ? "bg-red-100 text-red-600 hover:bg-red-200" : "bg-green-100 text-green-600 hover:bg-green-200"
                        )}
                    >
                        {isPlaying ? <Pause className="w-5 h-5" /> : <Play className="w-5 h-5" />}
                    </button>
                    <div className="text-sm font-medium text-gray-700">
                        {currentIndex + 1} / {events.length}
                    </div>
                </div>

                <div className="flex items-center space-x-4 flex-1 mx-4">
                    <input
                        type="range"
                        min="0"
                        max={events.length - 1}
                        value={currentIndex}
                        onChange={(e) => handleStep(parseInt(e.target.value))}
                        className="w-full h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer accent-blue-600"
                    />
                </div>

                <select
                    value={playbackSpeed}
                    onChange={(e) => setPlaybackSpeed(Number(e.target.value))}
                    className="text-xs border-gray-300 rounded-md shadow-sm focus:border-blue-500 focus:ring-blue-500"
                >
                    <option value="0.5">0.5x</option>
                    <option value="1">1x</option>
                    <option value="2">2x</option>
                    <option value="4">4x</option>
                </select>
            </div>

            {/* Split View: List & Detail */}
            <div className="flex-1 min-h-0 flex flex-col md:flex-row">
                {/* Event List (Vertical Timeline) */}
                <div className="w-full md:w-1/3 border-r border-gray-200 overflow-y-auto bg-gray-50 p-4" ref={scrollRef}>
                    <div className="space-y-4 relative before:absolute before:inset-0 before:ml-5 before:-translate-x-px md:before:mx-auto md:before:translate-x-0 before:h-full before:w-0.5 before:bg-gradient-to-b before:from-transparent before:via-slate-300 before:to-transparent">
                        {events.map((event, idx) => (
                            <div
                                key={event.id}
                                onClick={() => handleStep(idx)}
                                className={clsx(
                                    "relative flex items-center justify-between md:justify-normal md:odd:flex-row-reverse group is-active cursor-pointer p-2 rounded-lg transition-all",
                                    currentIndex === idx ? "bg-blue-50 scale-105 shadow-sm ring-1 ring-blue-200" : "hover:bg-gray-100"
                                )}
                            >
                                {/* Icon / Dot */}
                                <div className={clsx(
                                    "flex items-center justify-center w-8 h-8 rounded-full border-2 shrink-0 z-10 md:order-1 md:group-odd:-translate-x-1/2 md:group-even:translate-x-1/2 shadow-sm",
                                    currentIndex === idx ? "bg-white border-blue-500" : "bg-gray-100 border-gray-300"
                                )}>
                                    {getIcon(event.type)}
                                </div>

                                {/* Content Card */}
                                <div className={clsx(
                                    "ml-4 md:ml-0 md:w-[calc(100%-2.5rem)] md:p-2",
                                    "md:group-odd:pr-6 md:group-even:pl-6 text-left"
                                )}>
                                    <div className="flex items-center justify-between">
                                        <time className="text-[10px] font-medium text-gray-500">{formatTime(event.timestamp)}</time>
                                        <span className="text-[10px] uppercase font-bold text-gray-400">{event.type}</span>
                                    </div>
                                    <h3 className="text-xs font-semibold text-gray-800 line-clamp-1">{event.title}</h3>
                                </div>
                            </div>
                        ))}
                    </div>
                </div>

                {/* Detail View */}
                <div className="flex-1 p-6 overflow-y-auto bg-white">
                    {currentEvent ? (
                        <div className="max-w-3xl mx-auto animate-in fade-in slide-in-from-bottom-2 duration-300">
                            <div className="flex items-center space-x-3 mb-6">
                                <div className="p-2 bg-gray-100 rounded-lg">
                                    {getIcon(currentEvent.type)}
                                </div>
                                <div>
                                    <h2 className="text-xl font-bold text-gray-900">{currentEvent.title}</h2>
                                    <p className="text-sm text-gray-500">{formatTime(currentEvent.timestamp)} • ID: {currentEvent.id}</p>
                                </div>
                            </div>

                            <div className="prose prose-sm max-w-none">
                                <div className="bg-gray-900 rounded-lg p-4 overflow-x-auto shadow-inner">
                                    <pre className="text-gray-100 font-mono text-sm whitespace-pre-wrap">
                                        {currentEvent.content}
                                    </pre>
                                </div>
                            </div>

                            {currentEvent.duration && (
                                <div className="mt-6 flex items-center text-sm text-gray-500">
                                    <Clock className="w-4 h-4 mr-2" />
                                    <span>Duration: {currentEvent.duration}ms</span>
                                </div>
                            )}
                        </div>
                    ) : (
                        <div className="flex flex-col items-center justify-center h-full text-gray-400">
                            <Clock className="w-12 h-12 mb-4 opacity-50" />
                            <p>Select an event to view details</p>
                        </div>
                    )}
                </div>
            </div>
        </div>
    );
}
