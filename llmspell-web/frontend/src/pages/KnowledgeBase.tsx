import { useState } from 'react';
import clsx from 'clsx';
import {
    FileText,
    Search,
    Upload,
    Trash2,
    File,
    Loader2,
    CheckCircle,
    AlertCircle,
    Database
} from 'lucide-react';
import type { RagDocument, VectorSearchResult } from '../api/types';

// Mock Data
const MOCK_DOCS: RagDocument[] = [
    { id: '1', filename: 'architecture_v2.pdf', type: 'pdf', size: 1024 * 1024 * 2.5, status: 'indexed', uploaded_at: '2023-10-25T10:00:00Z' },
    { id: '2', filename: 'meeting_notes_nov.md', type: 'md', size: 15 * 1024, status: 'indexed', uploaded_at: '2023-11-01T14:30:00Z' },
    { id: '3', filename: 'project_requirements.txt', type: 'txt', size: 5 * 1024, status: 'processing', uploaded_at: '2023-12-05T09:15:00Z' },
];

const MOCK_RESULTS: VectorSearchResult[] = [
    {
        id: 'c1',
        content: "The system should support modular storage backends including SQLite and vector extensions.",
        score: 0.92,
        metadata: { document_id: '1', filename: 'architecture_v2.pdf', chunk_index: 42 }
    },
    {
        id: 'c2',
        content: "All storage components have been consolidated into llmspell-storage for better maintainability.",
        score: 0.88,
        metadata: { document_id: '1', filename: 'architecture_v2.pdf', chunk_index: 43 }
    },
    {
        id: 'c3',
        content: "Requirements for Phase 13c include unifying the trait system across all crates.",
        score: 0.75,
        metadata: { document_id: '3', filename: 'project_requirements.txt', chunk_index: 5 }
    }
];

export function KnowledgeBase() {
    const [activeTab, setActiveTab] = useState<'documents' | 'search'>('documents');
    const [documents, setDocuments] = useState<RagDocument[]>(MOCK_DOCS);
    const [searchQuery, setSearchQuery] = useState('');
    const [searchResults, setSearchResults] = useState<VectorSearchResult[]>([]);
    const [isSearching, setIsSearching] = useState(false);
    const [isUploading, setIsUploading] = useState(false);

    // Simulate search
    const handleSearch = (e: React.FormEvent) => {
        e.preventDefault();
        if (!searchQuery.trim()) return;

        setIsSearching(true);
        // Mock API call
        setTimeout(() => {
            setSearchResults(MOCK_RESULTS);
            setIsSearching(false);
        }, 800);
    };

    // Simulate upload
    const handleFileUpload = (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;

        setIsUploading(true);

        // Mock API latency
        setTimeout(() => {
            const newDoc: RagDocument = {
                id: Math.random().toString(36).substr(2, 9),
                filename: file.name,
                type: file.name.split('.').pop() as any || 'txt',
                size: file.size,
                status: 'processing',
                uploaded_at: new Date().toISOString()
            };
            setDocuments([newDoc, ...documents]);
            setIsUploading(false);

            // Reset input
            e.target.value = '';

            // Simulate processing completion
            setTimeout(() => {
                setDocuments(prev => prev.map(d =>
                    d.id === newDoc.id ? { ...d, status: 'indexed' } : d
                ));
            }, 3000);
        }, 1500);
    };

    const formatBytes = (bytes: number) => {
        if (bytes === 0) return '0 Bytes';
        const k = 1024;
        const size = ['Bytes', 'KB', 'MB', 'GB'];
        const i = Math.floor(Math.log(bytes) / Math.log(k));
        return parseFloat((bytes / Math.pow(k, i)).toFixed(2)) + ' ' + size[i];
    };

    return (
        <div className="p-6 max-w-7xl mx-auto space-y-6">
            <div className="flex items-center justify-between">
                <div>
                    <h1 className="text-2xl font-bold text-gray-900 flex items-center gap-3">
                        <Database className="w-8 h-8 text-blue-600" />
                        Knowledge Base
                    </h1>
                    <p className="mt-1 text-sm text-gray-500">
                        Manage your RAG documents and test vector retrieval.
                    </p>
                </div>

                {activeTab === 'documents' && (
                    <>
                        <input
                            type="file"
                            className="hidden"
                            id="file-upload"
                            onChange={handleFileUpload}
                            disabled={isUploading}
                        />
                        <button
                            onClick={() => document.getElementById('file-upload')?.click()}
                            disabled={isUploading}
                            className="inline-flex items-center px-4 py-2 border border-transparent rounded-md shadow-sm text-sm font-medium text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:opacity-50"
                        >
                            {isUploading ? (
                                <Loader2 className="w-4 h-4 mr-2 animate-spin" />
                            ) : (
                                <Upload className="w-4 h-4 mr-2" />
                            )}
                            Upload Document
                        </button>
                    </>
                )}
            </div>

            {/* Tabs */}
            <div className="border-b border-gray-200">
                <nav className="-mb-px flex space-x-8">
                    <button
                        onClick={() => setActiveTab('documents')}
                        className={clsx(
                            "whitespace-nowrap pb-4 px-1 border-b-2 font-medium text-sm flex items-center",
                            activeTab === 'documents'
                                ? "border-blue-500 text-blue-600"
                                : "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                        )}
                    >
                        <FileText className="w-4 h-4 mr-2" />
                        Documents
                        <span className="ml-2 bg-gray-100 text-gray-600 py-0.5 px-2.5 rounded-full text-xs">
                            {documents.length}
                        </span>
                    </button>
                    <button
                        onClick={() => setActiveTab('search')}
                        className={clsx(
                            "whitespace-nowrap pb-4 px-1 border-b-2 font-medium text-sm flex items-center",
                            activeTab === 'search'
                                ? "border-blue-500 text-blue-600"
                                : "border-transparent text-gray-500 hover:text-gray-700 hover:border-gray-300"
                        )}
                    >
                        <Search className="w-4 h-4 mr-2" />
                        Vector Explorer
                    </button>
                </nav>
            </div>

            {/* Content */}
            <div className="min-h-[500px]">
                {activeTab === 'documents' ? (
                    <div className="bg-white shadow overflow-hidden sm:rounded-lg">
                        <table className="min-w-full divide-y divide-gray-200">
                            <thead className="bg-gray-50">
                                <tr>
                                    <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        Name
                                    </th>
                                    <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        Type
                                    </th>
                                    <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        Size
                                    </th>
                                    <th scope="col" className="px-6 py-3 text-left text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        Status
                                    </th>
                                    <th scope="col" className="px-6 py-3 text-right text-xs font-medium text-gray-500 uppercase tracking-wider">
                                        Uploaded
                                    </th>
                                    <th scope="col" className="relative px-6 py-3">
                                        <span className="sr-only">Actions</span>
                                    </th>
                                </tr>
                            </thead>
                            <tbody className="bg-white divide-y divide-gray-200">
                                {documents.map((doc) => (
                                    <tr key={doc.id}>
                                        <td className="px-6 py-4 whitespace-nowrap">
                                            <div className="flex items-center">
                                                <div className="flex-shrink-0 h-10 w-10 flex items-center justify-center bg-gray-100 rounded-lg text-gray-500">
                                                    <File className="w-5 h-5" />
                                                </div>
                                                <div className="ml-4">
                                                    <div className="text-sm font-medium text-gray-900">{doc.filename}</div>
                                                    <div className="text-xs text-gray-400 font-mono">ID: {doc.id}</div>
                                                </div>
                                            </div>
                                        </td>
                                        <td className="px-6 py-4 whitespace-nowrap">
                                            <span className="text-sm text-gray-900 uppercase bg-gray-100 px-2 py-1 rounded text-xs font-semibold">
                                                {doc.type}
                                            </span>
                                        </td>
                                        <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-500">
                                            {formatBytes(doc.size)}
                                        </td>
                                        <td className="px-6 py-4 whitespace-nowrap">
                                            <span className={clsx(
                                                "px-2 inline-flex text-xs leading-5 font-semibold rounded-full items-center",
                                                doc.status === 'indexed' ? "bg-green-100 text-green-800" :
                                                    doc.status === 'processing' ? "bg-blue-100 text-blue-800" :
                                                        "bg-gray-100 text-gray-800"
                                            )}>
                                                {doc.status === 'indexed' && <CheckCircle className="w-3 h-3 mr-1" />}
                                                {doc.status === 'processing' && <Loader2 className="w-3 h-3 mr-1 animate-spin" />}
                                                {doc.status === 'failed' && <AlertCircle className="w-3 h-3 mr-1" />}
                                                {doc.status.charAt(0).toUpperCase() + doc.status.slice(1)}
                                            </span>
                                        </td>
                                        <td className="px-6 py-4 whitespace-nowrap text-right text-sm text-gray-500">
                                            {new Date(doc.uploaded_at).toLocaleDateString()}
                                        </td>
                                        <td className="px-6 py-4 whitespace-nowrap text-right text-sm font-medium">
                                            <button className="text-red-600 hover:text-red-900">
                                                <Trash2 className="w-4 h-4" />
                                            </button>
                                        </td>
                                    </tr>
                                ))}
                            </tbody>
                        </table>
                    </div>
                ) : (
                    <div className="space-y-6">
                        <div className="bg-white p-6 shadow rounded-lg">
                            <form onSubmit={handleSearch} className="relative">
                                <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                                    <Search className="h-5 w-5 text-gray-400" />
                                </div>
                                <input
                                    type="text"
                                    className="block w-full pl-10 pr-3 py-3 border border-gray-300 rounded-lg leading-5 bg-white placeholder-gray-500 focus:outline-none focus:placeholder-gray-400 focus:ring-1 focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                                    placeholder="Enter your query to search the knowledge base..."
                                    value={searchQuery}
                                    onChange={(e) => setSearchQuery(e.target.value)}
                                />
                                <button
                                    type="submit"
                                    disabled={isSearching}
                                    className="absolute inset-y-2 right-2 px-4 flex items-center bg-blue-600 text-white rounded-md text-sm font-medium hover:bg-blue-700 disabled:opacity-50"
                                >
                                    {isSearching ? <Loader2 className="w-4 h-4 animate-spin" /> : 'Search'}
                                </button>
                            </form>
                        </div>

                        {searchResults.length > 0 && (
                            <div className="space-y-4">
                                {searchResults.map((result) => (
                                    <div key={result.id} className="bg-white shadow rounded-lg p-5 border-l-4 border-blue-500 hover:shadow-md transition-shadow">
                                        <div className="flex justify-between items-start mb-2">
                                            <div className="flex items-center space-x-2">
                                                <span className="text-xs font-semibold px-2 py-1 bg-blue-50 text-blue-700 rounded">
                                                    Score: {(result.score * 100).toFixed(0)}%
                                                </span>
                                                <span className="text-sm text-gray-500 flex items-center">
                                                    <File className="w-3 h-3 mr-1" />
                                                    {result.metadata.filename}
                                                </span>
                                            </div>
                                        </div>
                                        <p className="text-gray-800 text-sm leading-relaxed">
                                            "{result.content}"
                                        </p>
                                    </div>
                                ))}
                            </div>
                        )}

                        {searchResults.length === 0 && !isSearching && searchQuery && (
                            <div className="text-center py-10 text-gray-500">
                                No results found for "{searchQuery}".
                            </div>
                        )}
                    </div>
                )}
            </div>
        </div>
    );
}
