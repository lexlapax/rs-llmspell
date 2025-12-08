import { useParams, Link } from 'react-router-dom';
import { useState, useEffect } from 'react';
import { WorkflowGraph } from '../components/workflow/WorkflowGraph';
import type { SessionDetails as SessionDetailsType, WorkflowExecution, WorkflowNode } from '../api/types';
import { ArrowLeft, Clock, Activity, CheckCircle } from 'lucide-react';

const MOCK_WORKFLOW: WorkflowExecution = {
    nodes: [
        { id: '1', label: 'Research Topic', type: 'agent', status: 'completed' },
        { id: '2', label: 'Search Web', type: 'tool', status: 'completed' },
        { id: '3', label: 'Fetch Papers', type: 'tool', status: 'completed' },
        { id: '4', label: 'Outline Draft', type: 'agent', status: 'completed' },
        { id: '5', label: 'Write Sections', type: 'agent', status: 'running' },
        { id: '6', label: 'Review Grammar', type: 'tool', status: 'pending' },
        { id: '7', label: 'Final Polish', type: 'agent', status: 'pending' },
    ],
    links: [
        { source: '1', target: '2' },
        { source: '1', target: '3' },
        { source: '2', target: '4' },
        { source: '3', target: '4' },
        { source: '4', target: '5' },
        { source: '5', target: '6' },
        { source: '6', target: '7' },
    ]
};

const MOCK_SESSION: SessionDetailsType = {
    id: 'mock-session-123',
    template_id: 'research-assistant',
    created_at: new Date().toISOString(),
    status: 'running',
    workflow: MOCK_WORKFLOW,
    logs: [
        '[INFO] Starting Research Assistant...',
        '[INFO] Agent "Research Topic" initialized.',
        '[INFO] Executing tool "Search Web"... found 12 results.',
        '[INFO] Executing tool "Fetch Papers"... downloaded 3 PDFs.',
        '[INFO] Agent "Outline Draft" completed.',
        '[INFO] Agent "Write Sections" started...',
    ],
    metadata: {
        provider_name: 'ollama',
        model: 'llama3:8b',
        max_tokens: 4000
    }
};

export function SessionDetails() {
    const { id } = useParams<{ id: string }>();
    const [session, setSession] = useState<SessionDetailsType | null>(null);
    const [selectedNode, setSelectedNode] = useState<WorkflowNode | null>(null);

    useEffect(() => {
        // Mock fetch
        setTimeout(() => {
            setSession(MOCK_SESSION);
        }, 500);
    }, [id]);

    if (!session) {
        return <div className="p-8 text-center text-gray-500">Loading session details...</div>;
    }

    return (
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8 h-[calc(100vh-64px)] flex flex-col">
            {/* Header */}
            <div className="flex items-center justify-between mb-6">
                <div className="flex items-center space-x-4">
                    <Link to="/sessions" className="text-gray-500 hover:text-gray-700">
                        <ArrowLeft className="h-6 w-6" />
                    </Link>
                    <div>
                        <h1 className="text-2xl font-bold text-gray-900 flex items-center gap-2">
                            Research Session
                            <span className="text-sm font-normal text-gray-400 px-2 py-0.5 bg-gray-100 rounded-full font-mono">{session.id}</span>
                        </h1>
                        <div className="flex items-center text-sm text-gray-500 mt-1 space-x-4">
                            <span className="flex items-center"><Clock className="h-4 w-4 mr-1" /> {new Date(session.created_at).toLocaleString()}</span>
                            <span className="flex items-center capitalize">
                                {session.status === 'running' && <Activity className="h-4 w-4 mr-1 text-blue-500" />}
                                {session.status === 'completed' && <CheckCircle className="h-4 w-4 mr-1 text-green-500" />}
                                {session.status}
                            </span>
                        </div>
                    </div>
                </div>
                <div className="flex space-x-3">
                    {/* Actions if needed */}
                </div>
            </div>

            {/* Content Layout */}
            <div className="flex-1 grid grid-cols-1 lg:grid-cols-3 gap-6 min-h-0">

                {/* Visualizer Column (2/3 width) */}
                <div className="lg:col-span-2 flex flex-col bg-white rounded-lg shadow min-h-[400px]">
                    <div className="p-4 border-b border-gray-100 flex justify-between items-center">
                        <h3 className="font-medium text-gray-900 flex items-center gap-2"><Activity className="h-4 w-4 text-blue-500" /> Workflow Visualization</h3>
                        <span className="text-xs text-gray-400">DAG View</span>
                    </div>
                    <div className="flex-1 relative bg-gray-50 m-2 rounded border border-gray-100 overflow-hidden">
                        <WorkflowGraph
                            data={session.workflow}
                            onNodeClick={setSelectedNode}
                        />
                    </div>
                </div>

                {/* Details Column (1/3 width) */}
                <div className="flex flex-col gap-6 overflow-hidden">
                    {/* Node Inspector */}
                    <div className="bg-white rounded-lg shadow p-4 flex-1 overflow-y-auto">
                        <h3 className="font-medium text-gray-900 mb-4 border-b pb-2">
                            {selectedNode ? `Step: ${selectedNode.label} ` : 'Step Details'}
                        </h3>
                        {selectedNode ? (
                            <div className="space-y-3 text-sm">
                                <div><span className="text-gray-500">ID:</span> <span className="font-mono">{selectedNode.id}</span></div>
                                <div><span className="text-gray-500">Type:</span> <span className="capitalize">{selectedNode.type}</span></div>
                                <div>
                                    <span className="text-gray-500">Status:</span>
                                    <span className={`ml - 2 px - 2 py - 0.5 rounded - full text - xs font - medium capitalize 
                                        ${selectedNode.status === 'completed' ? 'bg-green-100 text-green-800' :
                                            selectedNode.status === 'running' ? 'bg-blue-100 text-blue-800' :
                                                selectedNode.status === 'failed' ? 'bg-red-100 text-red-800' : 'bg-gray-100 text-gray-800'
                                        } `}>
                                        {selectedNode.status}
                                    </span>
                                </div>
                                {/* Placeholder for more data */}
                                <div className="mt-4 p-3 bg-gray-50 rounded border border-gray-100">
                                    <p className="text-gray-500 italic">Output logs would appear here...</p>
                                </div>
                            </div>
                        ) : (
                            <p className="text-gray-500 text-sm">Select a node in the graph to view details.</p>
                        )}
                    </div>

                    {/* Session Log Stream */}
                    <div className="bg-black rounded-lg shadow p-4 h-1/2 flex flex-col text-xs font-mono text-gray-300">
                        <div className="mb-2 text-gray-500 font-sans font-medium uppercase tracking-wider text-[10px]">System Logs</div>
                        <div className="overflow-y-auto flex-1 space-y-1">
                            {session.logs.map((log, i) => (
                                <div key={i} className="hover:text-white transition-colors">{log}</div>
                            ))}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
