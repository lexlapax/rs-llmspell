import { useRef, useState, useCallback, useEffect } from 'react';
import ForceGraph2D, { type ForceGraphMethods, type NodeObject, type LinkObject } from 'react-force-graph-2d';

interface GraphNode extends NodeObject {
    id: string;
    group: number;
    val: number; // radius
    label?: string;
}

interface GraphLink extends LinkObject {
    source: string | GraphNode;
    target: string | GraphNode;
    value: number; // thickness
}

interface GraphData {
    nodes: GraphNode[];
    links: GraphLink[];
}

export default function MemoryGraph() {
    const fgRef = useRef<ForceGraphMethods | undefined>(undefined);
    const containerRef = useRef<HTMLDivElement>(null);
    const [dimensions, setDimensions] = useState({ width: 800, height: 600 });
    const [data, setData] = useState<GraphData>({ nodes: [], links: [] });

    // Initial mock data generation
    useEffect(() => {
        // Initial "seed" nodes
        const nodes: GraphNode[] = [
            { id: 'User', group: 1, val: 20, label: 'User' },
            { id: 'LLMSpell', group: 1, val: 20, label: 'Assistant' },
            { id: 'Rust', group: 2, val: 10, label: 'Topic' },
            { id: 'Memory', group: 2, val: 10, label: 'Topic' },
            { id: 'Graph', group: 2, val: 10, label: 'Topic' },
            { id: 'Phase-14', group: 3, val: 15, label: 'Task' },
        ];

        const links: GraphLink[] = [
            { source: 'User', target: 'LLMSpell', value: 2 },
            { source: 'LLMSpell', target: 'Rust', value: 1 },
            { source: 'LLMSpell', target: 'Memory', value: 1 },
            { source: 'Memory', target: 'Graph', value: 3 },
            { source: 'LLMSpell', target: 'Phase-14', value: 3 },
            { source: 'Phase-14', target: 'Graph', value: 1 },
        ];

        setData({ nodes, links });
    }, []);

    // Handle resize
    useEffect(() => {
        const updateDimensions = () => {
            if (containerRef.current) {
                setDimensions({
                    width: containerRef.current.clientWidth,
                    height: containerRef.current.clientHeight
                });
            }
        };

        window.addEventListener('resize', updateDimensions);
        updateDimensions(); // Initial call

        return () => window.removeEventListener('resize', updateDimensions);
    }, []);

    const handleNodeClick = useCallback((node: NodeObject) => {
        const gNode = node as GraphNode;
        console.log('Clicked node:', gNode);

        // Zoom to node on click
        if (fgRef.current) {
            fgRef.current.centerAt(gNode.x, gNode.y, 1000);
            fgRef.current.zoom(4, 2000);
        }
    }, []);

    const simulateTraffic = useCallback(() => {
        setData(prevData => {
            const newNodes = [...prevData.nodes];
            const newLinks = [...prevData.links];

            const id = `Node-${newNodes.length}`;
            const targetNode = newNodes[Math.floor(Math.random() * newNodes.length)];

            newNodes.push({
                id,
                group: Math.floor(Math.random() * 5),
                val: Math.random() * 10 + 5,
                label: id
            });

            newLinks.push({
                source: id,
                target: targetNode.id,
                value: 1
            });

            return { nodes: newNodes, links: newLinks };
        });
    }, []);

    return (
        <div className="flex flex-col h-full bg-slate-900 rounded-lg overflow-hidden border border-slate-700 shadow-xl">
            <div className="flex items-center justify-between px-4 py-3 bg-slate-800 border-b border-slate-700 shrink-0">
                <h2 className="text-sm font-medium text-white flex items-center">
                    <span className="w-2 h-2 rounded-full bg-blue-500 mr-2"></span>
                    Semantic Memory Graph (Mock Data)
                </h2>
                <div className="flex space-x-2">
                    <button
                        onClick={simulateTraffic}
                        className="px-3 py-1 text-xs font-medium text-white bg-blue-600 rounded hover:bg-blue-700 transition-colors"
                    >
                        Add Node
                    </button>
                    <button
                        onClick={() => {
                            if (fgRef.current) {
                                fgRef.current.zoomToFit(400);
                            }
                        }}
                        className="px-3 py-1 text-xs font-medium text-slate-300 bg-slate-700 rounded hover:bg-slate-600 transition-colors"
                    >
                        Reset View
                    </button>
                </div>
            </div>

            <div ref={containerRef} className="flex-1 min-h-0 relative">
                <ForceGraph2D
                    ref={fgRef}
                    width={dimensions.width}
                    height={dimensions.height}
                    graphData={data}
                    nodeLabel="label"
                    nodeColor={(node: any) => {
                        const colors = ['#60a5fa', '#34d399', '#f472b6', '#fbbf24', '#a78bfa'];
                        return colors[node.group % colors.length];
                    }}
                    linkColor={() => '#475569'}
                    backgroundColor="#0f172a"
                    onNodeClick={handleNodeClick}
                    nodeRelSize={6}
                    linkWidth={link => (link as GraphLink).value}
                    linkDirectionalParticles={2}
                    linkDirectionalParticleSpeed={0.005}
                />
            </div>
        </div>
    );
}
