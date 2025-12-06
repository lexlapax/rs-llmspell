import { useState, useEffect } from 'react';
import { api } from '../api/client';
import type { Template, TemplateDetails } from '../api/types';
import { TemplateCard } from '../components/templates/TemplateCard';
import { LaunchModal } from '../components/templates/LaunchModal';
import { Search, Loader2, Book } from 'lucide-react';
import clsx from 'clsx';
import { useNavigate } from 'react-router-dom';

const CATEGORIES = ['All', 'Research', 'Chat', 'Data', 'Code', 'Workflow'];

export function Templates() {
    const navigate = useNavigate();
    const [templates, setTemplates] = useState<Template[]>([]);
    const [isLoading, setIsLoading] = useState(true);
    const [searchQuery, setSearchQuery] = useState('');
    const [selectedCategory, setSelectedCategory] = useState('All');

    // Modal state
    const [isModalOpen, setIsModalOpen] = useState(false);
    const [selectedTemplate, setSelectedTemplate] = useState<TemplateDetails | null>(null);

    // Mock Data
    const MOCK_TEMPLATES: Template[] = [
        {
            id: 'tmpl_research_v1',
            name: 'Deep Research Assistant',
            description: 'Automated research agent that searches the web, summarizes findings, and compiles a report with citations.',
            category: 'Research',
            tags: ['research', 'web-search', 'automation'],
            version: '1.0.0',
            config_schema: {}
        },
        {
            id: 'tmpl_chat_v2',
            name: 'Advanced Chatbot',
            description: 'A highly capable conversational agent with long-term memory and context awareness.',
            category: 'Chat',
            tags: ['chat', 'memory', 'nlp'],
            version: '2.1.0',
            config_schema: {}
        },
        {
            id: 'tmpl_rust_coder',
            name: 'Rust Code Architect',
            description: 'Specialized assistant for generating, refactoring, and documenting Rust codebases.',
            category: 'Code',
            tags: ['rust', 'coding', 'development'],
            version: '1.2.0',
            config_schema: {}
        },
        {
            id: 'tmpl_data_etl',
            name: 'Data ETL Pipeline',
            description: 'Extract, Transform, and Load textual data into structured formats for analysis.',
            category: 'Data',
            tags: ['data', 'etl', 'processing'],
            version: '0.9.5',
            config_schema: {}
        },
        {
            id: 'tmpl_workflow_gen',
            name: 'Workflow Generator',
            description: 'Create multi-step agent workflows from natural language descriptions.',
            category: 'Workflow',
            tags: ['workflow', 'orchestration'],
            version: '1.0.0',
            config_schema: {}
        }
    ];

    const MOCK_DETAILS: TemplateDetails = {
        metadata: MOCK_TEMPLATES[0] as any,
        schema: {
            version: "1.0",
            parameters: [
                {
                    name: "agent_name",
                    description: "Name of the agent to deploy",
                    type: "string",
                    required: true,
                    default: "My Agent"
                },
                {
                    name: "model",
                    description: "LLM Model to use",
                    type: "string",
                    required: true,
                    default: "gpt-4",
                    constraints: { allowed_values: ["gpt-4", "gpt-3.5-turbo", "claude-2"] }
                }
            ]
        }
    };

    useEffect(() => {
        loadTemplates();
    }, []);

    const loadTemplates = async () => {
        setIsLoading(true);
        try {
            const data = await api.getTemplates();
            if (data && data.length > 0) {
                setTemplates(data);
            } else {
                console.log('Using mock templates (empty API response)');
                setTemplates(MOCK_TEMPLATES);
            }
        } catch (error) {
            console.warn('Backend unreachable, using mock templates:', error);
            setTemplates(MOCK_TEMPLATES);
        } finally {
            setIsLoading(false);
        }
    };

    const handleLaunchClick = async (template: Template) => {
        try {
            try {
                const details = await api.getTemplate(template.id);
                setSelectedTemplate(details);
            } catch (e) {
                console.warn('Failed to fetch details, using mock:', e);
                // Fallback details logic
                setSelectedTemplate({
                    metadata: template as any,
                    schema: MOCK_DETAILS.schema
                });
            }
            setIsModalOpen(true);
        } catch (error) {
            console.error('Error opening modal:', error);
        }
    };

    const handleLaunchConfirm = async (id: string, config: Record<string, any>) => {
        await api.launchTemplate(id, config);
        // Navigate to the new session or show success
        // For now, let's navigate to sessions page
        navigate('/sessions');
    };

    const getCategoryLabel = (cat: string | Record<string, string>) => {
        if (typeof cat === 'string') return cat;
        if (cat.Custom) return cat.Custom;
        return Object.keys(cat)[0];
    };

    const filteredTemplates = templates.filter(t => {
        const catLabel = getCategoryLabel(t.category);
        const matchesCategory = selectedCategory === 'All' || catLabel.toLowerCase() === selectedCategory.toLowerCase();
        const matchesSearch = t.name.toLowerCase().includes(searchQuery.toLowerCase()) ||
            t.description.toLowerCase().includes(searchQuery.toLowerCase());
        return matchesCategory && matchesSearch;
    });

    if (isLoading) {
        return (
            <div className="flex h-full items-center justify-center">
                <Loader2 className="h-8 w-8 animate-spin text-blue-500" />
            </div>
        );
    }

    return (
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-8">
            <div className="flex justify-between items-center mb-8">
                <div>
                    <h1 className="text-2xl font-bold text-gray-900">Template Library</h1>
                    <p className="mt-1 text-sm text-gray-500">
                        Launch specialized AI workflows and agents.
                    </p>
                </div>
            </div>

            {/* Filters and Search */}
            <div className="flex flex-col sm:flex-row justify-between items-start sm:items-center gap-4 mb-8">
                <div className="flex flex-wrap gap-2">
                    {CATEGORIES.map(category => (
                        <button
                            key={category}
                            onClick={() => setSelectedCategory(category)}
                            className={clsx(
                                "px-3 py-1.5 rounded-full text-sm font-medium transition-colors",
                                selectedCategory === category
                                    ? "bg-blue-100 text-blue-800"
                                    : "bg-gray-100 text-gray-600 hover:bg-gray-200"
                            )}
                        >
                            {category}
                        </button>
                    ))}
                </div>
                <div className="relative w-full sm:w-64">
                    <div className="absolute inset-y-0 left-0 pl-3 flex items-center pointer-events-none">
                        <Search className="h-4 w-4 text-gray-400" />
                    </div>
                    <input
                        type="text"
                        placeholder="Search templates..."
                        className="block w-full pl-10 pr-3 py-2 border border-gray-300 rounded-md leading-5 bg-white placeholder-gray-500 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                    />
                </div>
            </div>

            {filteredTemplates.length === 0 ? (
                <div className="text-center py-12 bg-gray-50 rounded-lg">
                    <Book className="mx-auto h-12 w-12 text-gray-400" />
                    <h3 className="mt-2 text-lg font-medium text-gray-900">No templates found</h3>
                    <p className="mt-1 text-sm text-gray-500">
                        Try adjusting your search or filters.
                    </p>
                </div>
            ) : (
                <div className="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3">
                    {filteredTemplates.map(template => (
                        <TemplateCard
                            key={template.id}
                            template={template}
                            onLaunch={handleLaunchClick}
                        />
                    ))}
                </div>
            )}

            {/* Launch Modal */}
            <LaunchModal
                isOpen={isModalOpen}
                onClose={() => setIsModalOpen(false)}
                template={selectedTemplate}
                onLaunch={handleLaunchConfirm}
            />
        </div>
    );
}
