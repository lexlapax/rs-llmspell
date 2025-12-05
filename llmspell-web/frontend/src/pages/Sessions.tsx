import { useState } from 'react';
import Timeline, { type TimelineEvent } from '../components/session/Timeline';
import { PlayCircle } from 'lucide-react';

// Mock Session Data
const MOCK_SESSIONS = [
    { id: 'sess_01', name: 'Refactor Auth', created_at: Date.now() - 3600000, status: 'completed' },
    { id: 'sess_02', name: 'Debug Memory', created_at: Date.now() - 7200000, status: 'failed' },
    { id: 'sess_03', name: 'Plan Phase 14', created_at: Date.now() - 86400000, status: 'active' },
];

const MOCK_EVENTS: TimelineEvent[] = [
    { id: 'evt_1', type: 'user', title: 'User Request', content: 'Refactor the authentication middleware to use JWTs.', timestamp: Date.now() - 3500000 },
    { id: 'evt_2', type: 'assistant', title: 'Plan Creation', content: 'Creating implementation plan for Auth Middleware...', timestamp: Date.now() - 3495000, duration: 250 },
    { id: 'evt_3', type: 'tool', title: 'File Read', content: 'Reading /src/middleware/auth.rs', timestamp: Date.now() - 3490000, duration: 120 },
    { id: 'evt_4', type: 'assistant', title: 'Code Generation', content: 'Generating JWT validation logic using `jsonwebtoken` crate.', timestamp: Date.now() - 3480000, duration: 1500 },
    { id: 'evt_5', type: 'system', title: 'Compilation', content: 'Compiling llmspell-web v0.1.0...', timestamp: Date.now() - 3475000, duration: 4200 },
    { id: 'evt_6', type: 'error', title: 'Test Failed', content: 'Error: Invalid signature key type. Expected RSA, got Ed25519.', timestamp: Date.now() - 3470000 },
    { id: 'evt_7', type: 'user', title: 'User Feedback', content: 'Fix the key type error. Use Ed25519.', timestamp: Date.now() - 3460000 },
    { id: 'evt_8', type: 'assistant', title: 'Fix Applied', content: 'Updating key parsing logic...', timestamp: Date.now() - 3450000, duration: 800 },
    { id: 'evt_9', type: 'system', title: 'Success', content: 'All tests passed.', timestamp: Date.now() - 3440000 },
];

export function Sessions() {
    const [selectedSessionId, setSelectedSessionId] = useState<string | null>(null);

    return (
        <div className="flex h-[calc(100vh-4rem)]">
            {/* Session List Sidebar */}
            <div className="w-80 border-r border-gray-200 bg-white flex flex-col shrink-0">
                <div className="p-4 border-b border-gray-200">
                    <h2 className="text-lg font-bold text-gray-900">Sessions</h2>
                </div>
                <div className="flex-1 overflow-y-auto">
                    {MOCK_SESSIONS.map(session => (
                        <div
                            key={session.id}
                            onClick={() => setSelectedSessionId(session.id)}
                            className={`p-4 border-b border-gray-100 cursor-pointer hover:bg-gray-50 transition-colors ${selectedSessionId === session.id ? 'bg-blue-50 border-l-4 border-l-blue-500' : ''}`}
                        >
                            <h3 className="font-medium text-gray-900">{session.name}</h3>
                            <div className="flex justify-between mt-2 text-xs text-gray-500">
                                <span>{new Date(session.created_at).toLocaleDateString()}</span>
                                <span className={`capitalize ${session.status === 'completed' ? 'text-green-600' : session.status === 'failed' ? 'text-red-600' : 'text-blue-600'}`}>
                                    {session.status}
                                </span>
                            </div>
                        </div>
                    ))}
                </div>
            </div>

            {/* Timeline View */}
            <div className="flex-1 flex flex-col min-w-0 bg-gray-50">
                {selectedSessionId ? (
                    <div className="flex-1 p-6 overflow-hidden flex flex-col">
                        <div className="mb-4 flex justify-between items-center shrink-0">
                            <div>
                                <h2 className="text-2xl font-bold text-gray-800">Session Replay</h2>
                                <p className="text-sm text-gray-500">ID: {selectedSessionId} • {MOCK_EVENTS.length} Events</p>
                            </div>
                        </div>
                        <div className="flex-1 min-h-0">
                            <Timeline events={MOCK_EVENTS} />
                        </div>
                    </div>
                ) : (
                    <div className="flex-1 flex flex-col items-center justify-center text-gray-400">
                        <PlayCircle className="w-16 h-16 mb-4 opacity-20" />
                        <p className="text-lg">Select a session to view timeline</p>
                    </div>
                )}
            </div>
        </div>
    );
}
