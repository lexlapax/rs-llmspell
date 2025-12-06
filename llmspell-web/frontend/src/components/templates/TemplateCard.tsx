
import type { Template } from '../../api/types';
import { Play, Tag } from 'lucide-react';

interface TemplateCardProps {
    template: Template;
    onLaunch: (template: Template) => void;
}

export function TemplateCard({ template, onLaunch }: TemplateCardProps) {
    const getCategoryLabel = (cat: string | Record<string, string>) => {
        if (typeof cat === 'string') return cat;
        if (cat.Custom) return cat.Custom;
        return Object.keys(cat)[0];
    };

    return (
        <div className="bg-white overflow-hidden shadow rounded-lg border border-gray-200 hover:shadow-md transition-shadow flex flex-col h-full">
            <div className="px-4 py-5 sm:p-6 flex-1 flex flex-col">
                <div className="flex items-center justify-between mb-4">
                    <span className="inline-flex items-center px-2.5 py-0.5 rounded-full text-xs font-medium bg-blue-100 text-blue-800 capitalize">
                        {getCategoryLabel(template.category)}
                    </span>
                    <span className="text-xs text-gray-400 font-mono">v{template.version}</span>
                </div>

                <h3 className="text-lg leading-6 font-medium text-gray-900 mb-2">
                    {template.name}
                </h3>

                <p className="text-sm text-gray-500 mb-4 flex-1 line-clamp-3">
                    {template.description}
                </p>

                <div className="flex items-center flex-wrap gap-2 mb-4">
                    {template.tags.map(tag => (
                        <span key={tag} className="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium bg-gray-100 text-gray-800">
                            <Tag className="w-3 h-3 mr-1 text-gray-400" />
                            {tag}
                        </span>
                    ))}
                </div>
            </div>

            <div className="bg-gray-50 px-4 py-4 sm:px-6 border-t border-gray-200">
                <button
                    onClick={() => onLaunch(template)}
                    className="w-full inline-flex justify-center items-center px-4 py-2 border border-transparent shadow-sm text-sm font-medium rounded-md text-white bg-blue-600 hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
                >
                    <Play className="w-4 h-4 mr-2" />
                    Launch
                </button>
            </div>
        </div>
    );
}
