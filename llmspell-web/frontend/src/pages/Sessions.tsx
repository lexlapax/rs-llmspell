import { useState } from 'react';
import { Link } from 'react-router-dom';
import Timeline, { type TimelineEvent } from '../components/session/Timeline';
import { PlayCircle } from 'lucide-react';

// Mock data remains the same
const MOCK_SESSIONS = [
    { id: 'sess_123', template: 'Research Assistant', status: 'completed', duration: '45s', created_at: '10 mins ago' },
    { id: 'sess_124', template: 'Code Generator', status: 'running', duration: '12s', created_at: '2 mins ago' },
    { id: 'sess_125', template: 'Data Analyst', status: 'failed', duration: '2s', created_at: '1 min ago' },
];

const MOCK_EVENTS: TimelineEvent[] = [
    { id: '1', timestamp: Date.now() - 10000, type: 'system', title: 'Session Started', content: 'Session initialized by user.' },
    { id: '2', timestamp: Date.now() - 8000, type: 'tool', title: 'Agent Init', content: 'Research Agent initialized with topic: "Rust async"' },
    { id: '3', timestamp: Date.now() - 5000, type: 'tool', title: 'Searching Web', content: 'Searching web for "Rust async"' },
    { id: '4', timestamp: Date.now() - 2000, type: 'assistant', title: 'Search Results', content: 'Found 5 relevant articles.' },
];

export function Sessions() {
    const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);

    return (
        <div className="flex h-[calc(100vh-4rem)]">
            <div className="w-1/3 border-r border-gray-200 bg-white overflow-y-auto">
                <div className="p-4 border-b border-gray-200">
                    <h2 className="text-lg font-semibold text-gray-900">Recent Sessions</h2>
                </div>
                <div className="divide-y divide-gray-200">
                    {MOCK_SESSIONS.map((session) => (
                        <div
                            key={session.id}
                            onClick={() => setSelectedSessionId(session.id)}
                            className={`p-4 hover:bg-gray-50 cursor-pointer transition-colors ${selectedSessionId === session.id ? 'bg-blue-50' : ''}`}
                        >
                            <div className="flex justify-between items-start mb-1">
                                <span className="font-medium text-gray-900">{session.template}</span>
                                <span className="text-xs text-gray-500">{session.created_at}</span>
                            </div>
                            <div className="flex justify-between items-center text-sm">
                                <span className="font-mono text-gray-500 text-xs">{session.id}</span>
                                <span className={`px-2 py-0.5 rounded-full text-xs font-medium capitalize 
                                    ${session.status === 'completed' ? 'bg-green-100 text-green-800' :
                                        session.status === 'running' ? 'bg-blue-100 text-blue-800' :
                                            'bg-red-100 text-red-800'}`}>
                                    {session.status}
                                </span>
                            </div>
                            <div className="mt-2 text-right">
                                <Link to={`/sessions/${session.id}`} className="text-blue-600 hover:text-blue-900 text-sm font-medium">View Details</Link>
                            </div>
                        </div>
                    ))}
                </div>
            </div>

            <div className="flex-1 flex flex-col bg-gray-50">
                {selectedSessionId ? (
                    <>
                        <div className="p-4 bg-white border-b border-gray-200 flex justify-between items-center shadow-sm">
                            <div>
                                <h2 className="text-2xl font-bold text-gray-800">Session Replay</h2>
                                <p className="text-sm text-gray-500">ID: {selectedSessionId} • {MOCK_EVENTS.length} Events</p>
                            </div>
                            <Link to={`/sessions/${selectedSessionId}`} className="text-blue-600 hover:text-blue-900 text-sm font-medium">View Details</Link>
                        </div>
                        <div className="flex-1 min-h-0">
                            <Timeline events={MOCK_EVENTS} />
                        </div>
                    </>
                ) : (
                    <div className="flex-1 flex flex-col items-center justify-center text-gray-400">
                        <PlayCircle className="h-16 w-16 mb-4 opacity-20" />
                        <p className="text-lg font-medium">Select a session to replay</p>
                    </div>
                )}
            </div>
        </div>
    );
}
