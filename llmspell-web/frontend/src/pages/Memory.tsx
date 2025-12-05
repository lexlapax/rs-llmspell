import MemoryGraph from '../components/memory/MemoryGraph';

export default function Memory() {
    return (
        <div className="p-6 h-[calc(100vh-4rem)] flex flex-col">
            <div className="flex justify-between items-center mb-6 shrink-0">
                <div>
                    <h1 className="text-2xl font-bold text-gray-900">Memory Graph</h1>
                    <p className="text-gray-500 mt-1">Explore semantic knowledge graph and relationships.</p>
                </div>
            </div>
            <div className="flex-1 min-h-0">
                <MemoryGraph />
            </div>
        </div>
    );
}
