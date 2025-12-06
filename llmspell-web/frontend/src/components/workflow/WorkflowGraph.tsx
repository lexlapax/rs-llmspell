import { useRef, useCallback, useEffect, useState } from 'react';
import ForceGraph2D, { type ForceGraphMethods } from 'react-force-graph-2d';
import type { WorkflowExecution, WorkflowNode } from '../../api/types';
import { useResizeObserver } from '../../hooks/useResizeObserver';

interface WorkflowGraphProps {
    data: WorkflowExecution;
    onNodeClick?: (node: WorkflowNode) => void;
}

export function WorkflowGraph({ data, onNodeClick }: WorkflowGraphProps) {
    const graphRef = useRef<ForceGraphMethods | undefined>(undefined);
    const containerRef = useRef<HTMLDivElement>(null);
    const { width, height } = useResizeObserver(containerRef);
    const [highlightNodes, setHighlightNodes] = useState(new Set<string>());
    const [hoverNode, setHoverNode] = useState<WorkflowNode | null>(null);

    // Initial centering
    useEffect(() => {
        if (graphRef.current) {
            graphRef.current.d3Force('charge')?.strength(-300);
            setTimeout(() => {
                graphRef.current?.zoomToFit(400, 50);
            }, 500);
        }
    }, [data, width, height]);

    const getNodeColor = (node: WorkflowNode) => {
        if (highlightNodes.has(node.id)) return '#F59E0B'; // Amber for highlight
        switch (node.status) {
            case 'completed': return '#10B981'; // Emerald
            case 'running': return '#3B82F6';   // Blue
            case 'failed': return '#EF4444';    // Red
            default: return '#9CA3AF';          // Gray
        }
    };

    const paintNode = useCallback((node: any, ctx: CanvasRenderingContext2D, globalScale: number) => {
        const label = node.label;
        const fontSize = 12 / globalScale * 1.5; // Scale text
        ctx.font = `${fontSize}px Sans-Serif`;
        const textWidth = ctx.measureText(label).width;
        const bckgDimensions = [textWidth, fontSize].map(n => n + fontSize * 0.2); // some padding

        // Node circle
        ctx.beginPath();
        const r = 5;
        ctx.arc(node.x, node.y, r, 0, 2 * Math.PI, false);
        ctx.fillStyle = getNodeColor(node);
        ctx.fill();

        // Hover box if hovered or highlighted
        if (node === hoverNode || highlightNodes.has(node.id)) {
            ctx.fillStyle = 'rgba(255, 255, 255, 0.9)';
            ctx.fillRect(node.x - bckgDimensions[0] / 2, node.y - bckgDimensions[1] / 2 - 10, bckgDimensions[0], bckgDimensions[1]);

            ctx.textAlign = 'center';
            ctx.textBaseline = 'middle';
            ctx.fillStyle = '#000';
            ctx.fillText(label, node.x, node.y - 10);
        } else {
            // Always show label below
            ctx.textAlign = 'center';
            ctx.textBaseline = 'top';
            ctx.fillStyle = '#374151'; // Gray-700
            ctx.fillText(label, node.x, node.y + 8);
        }
    }, [hoverNode, highlightNodes]);

    return (
        <div ref={containerRef} className="w-full h-full min-h-[400px] border border-gray-200 rounded-lg bg-gray-50 overflow-hidden relative">
            {width > 0 && height > 0 && (
                <ForceGraph2D
                    ref={graphRef}
                    width={width}
                    height={height}
                    graphData={data}
                    nodeLabel="label"
                    nodeRelSize={6}
                    linkColor={() => '#D1D5DB'} // Gray-300
                    linkDirectionalArrowLength={3.5}
                    linkDirectionalArrowRelPos={1}
                    linkCurvature={0.25}
                    onNodeClick={(node) => {
                        setHighlightNodes(new Set([node.id]));
                        if (onNodeClick) onNodeClick(node as WorkflowNode);
                    }}
                    onNodeHover={(node) => setHoverNode(node as WorkflowNode || null)}
                    nodeCanvasObject={paintNode}
                    // Basic DAG force layout simulation params
                    dagMode="lr"
                    dagLevelDistance={80}
                />
            )}
            <div className="absolute top-2 right-2 bg-white/90 p-2 rounded text-xs shadow-sm border border-gray-100 flex flex-col gap-1">
                <div className="flex items-center gap-2"><span className="w-2 h-2 rounded-full bg-emerald-500"></span> Completed</div>
                <div className="flex items-center gap-2"><span className="w-2 h-2 rounded-full bg-blue-500"></span> Running</div>
                <div className="flex items-center gap-2"><span className="w-2 h-2 rounded-full bg-gray-400"></span> Pending</div>
                <div className="flex items-center gap-2"><span className="w-2 h-2 rounded-full bg-red-500"></span> Failed</div>
            </div>
        </div>
    );
}
