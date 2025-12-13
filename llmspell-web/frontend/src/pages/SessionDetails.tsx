import { useParams, Link } from 'react-router-dom';
import { useState, useEffect } from 'react';
import { WorkflowGraph } from '../components/workflow/WorkflowGraph';
import type { SessionDetailsResponse, WorkflowNode } from '../api/types';
import { api } from '../api/client';
import { ArrowLeft, Clock, Activity, CheckCircle, Loader2, AlertCircle } from 'lucide-react';

export function SessionDetails() {
    const { id } = useParams<{ id: string }>();
    const [session, setSession] = useState<SessionDetailsResponse | null>(null);
    const [selectedNode, setSelectedNode] = useState<WorkflowNode | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);

    useEffect(() => {
        if (!id) return;

        let pollInterval: ReturnType<typeof setInterval>;
        let isPolling = true;

        const fetchData = async () => {
            try {
                const data = await api.getSessionDetails(id);
                setSession(() => {
                    // Only update if something changed to avoid re-renders if needed, 
                    // but React handles object identity diffs reasonably well.
                    // For logs, appending might be better but replacing is simpler for now.
                    return data;
                });
                setError(null);

                // Stop polling if completed or failed
                if (data.status === 'completed' || data.status === 'failed' || data.status === 'stopped') {
                    isPolling = false;
                    clearInterval(pollInterval);
                }
            } catch (err) {
                console.error('Failed to fetch session details:', err);
                if (isPolling) {
                    // Don't show full error if just one poll failed, unless we have no data
                    if (!session) {
                        setError('Failed to load session details. ' + (err instanceof Error ? err.message : String(err)));
                    }
                }
            } finally {
                setIsLoading(false);
            }
        };

        // Initial fetch
        fetchData();

        // Start polling
        pollInterval = setInterval(() => {
            if (isPolling) fetchData();
        }, 2000);

        return () => clearInterval(pollInterval);
    }, [id]);

    if (isLoading && !session) {
        return (
            <div className="flex h-[calc(100vh-64px)] items-center justify-center">
                <Loader2 className="h-8 w-8 animate-spin text-blue-500" />
            </div>
        );
    }

    if (error && !session) {
        return (
            <div className="flex h-[calc(100vh-64px)] items-center justify-center text-red-500">
                <AlertCircle className="h-6 w-6 mr-2" />
                {error}
            </div>
        );
    }

    if (!session) return null;

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
                            {session.name}
                            <span className="text-sm font-normal text-gray-400 px-2 py-0.5 bg-gray-100 rounded-full font-mono">{session.id}</span>
                        </h1>
                        <div className="flex items-center text-sm text-gray-500 mt-1 space-x-4">
                            <span className="flex items-center"><Clock className="h-4 w-4 mr-1" /> {new Date(session.created_at).toLocaleString()}</span>
                            <span className="flex items-center capitalize">
                                {session.status === 'running' && <Activity className="h-4 w-4 mr-1 text-blue-500" />}
                                {session.status === 'completed' && <CheckCircle className="h-4 w-4 mr-1 text-green-500" />}
                                {['failed', 'error'].includes(session.status) && <AlertCircle className="h-4 w-4 mr-1 text-red-500" />}
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
                        {session.workflow ? (
                            <WorkflowGraph
                                data={session.workflow}
                                onNodeClick={setSelectedNode}
                            />
                        ) : (
                            <div className="flex items-center justify-center h-full text-gray-400">
                                No workflow execution data available.
                            </div>
                        )}
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
                                    <span className={`ml-2 px-2 py-0.5 rounded-full text-xs font-medium capitalize 
                                        ${selectedNode.status === 'completed' ? 'bg-green-100 text-green-800' :
                                            selectedNode.status === 'running' ? 'bg-blue-100 text-blue-800' :
                                                selectedNode.status === 'failed' ? 'bg-red-100 text-red-800' : 'bg-gray-100 text-gray-800'
                                        } `}>
                                        {selectedNode.status}
                                    </span>
                                </div>
                                {selectedNode.duration_ms && (
                                    <div><span className="text-gray-500">Duration:</span> {selectedNode.duration_ms}ms</div>
                                )}

                                {selectedNode.error && (
                                    <div className="mt-2 p-2 bg-red-50 border border-red-100 rounded text-red-700">
                                        Error: {selectedNode.error}
                                    </div>
                                )}

                                {selectedNode.output && (
                                    <div className="mt-4">
                                        <div className="text-gray-500 mb-1">Output:</div>
                                        <div className="p-2 bg-gray-50 rounded border border-gray-100 font-mono text-xs overflow-x-auto">
                                            {typeof selectedNode.output === 'string'
                                                ? selectedNode.output
                                                : JSON.stringify(selectedNode.output, null, 2)}
                                        </div>
                                    </div>
                                )}
                            </div>
                        ) : (
                            <p className="text-gray-500 text-sm">Select a node in the graph to view details.</p>
                        )}
                    </div>

                    {/* Session Log Stream */}
                    <div className="bg-black rounded-lg shadow p-4 h-1/2 flex flex-col text-xs font-mono text-gray-300">
                        <div className="mb-2 text-gray-500 font-sans font-medium uppercase tracking-wider text-[10px]">System Logs</div>
                        <div className="overflow-y-auto flex-1 space-y-1">
                            {session.logs && session.logs.length > 0 ? session.logs.map((log, i) => (
                                <div key={i} className="hover:text-white transition-colors border-b border-gray-800 pb-0.5 mb-0.5 last:border-0">{log}</div>
                            )) : (
                                <div className="text-gray-600 italic">No logs available</div>
                            )}
                        </div>
                    </div>
                </div>
            </div>
        </div>
    );
}
