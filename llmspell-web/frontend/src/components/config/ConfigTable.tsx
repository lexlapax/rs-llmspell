
import React, { useState } from 'react';
import type { ConfigItem } from '../../api/types';
import { Search, Info, Edit2 } from 'lucide-react';
import clsx from 'clsx';

interface ConfigTableProps {
    items: ConfigItem[];
    onEdit: (item: ConfigItem) => void;
}

export const ConfigTable: React.FC<ConfigTableProps> = ({ items, onEdit }) => {
    const [search, setSearch] = useState('');

    const filteredItems = items.filter(item =>
        item.name.toLowerCase().includes(search.toLowerCase()) ||
        (item.config_path && item.config_path.toLowerCase().includes(search.toLowerCase())) ||
        item.description.toLowerCase().includes(search.toLowerCase()) ||
        item.category.toLowerCase().includes(search.toLowerCase())
    );

    return (
        <div className="flex flex-col gap-4">
            {/* Toolbar */}
            <div className="flex items-center gap-2">
                <div className="relative flex-1 max-w-md">
                    <Search className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-400" />
                    <input
                        type="text"
                        placeholder="Search configuration..."
                        value={search}
                        onChange={(e) => setSearch(e.target.value)}
                        className="w-full pl-9 pr-4 py-2 bg-gray-900 border border-gray-700 rounded-lg text-sm text-gray-200 focus:outline-none focus:border-blue-500 transition-colors"
                    />
                </div>
            </div>

            {/* Table */}
            <div className="border border-gray-800 rounded-lg overflow-hidden bg-gray-900/50">
                <table className="w-full text-left text-sm">
                    <thead className="bg-gray-800 text-gray-400">
                        <tr>
                            <th className="px-4 py-3 font-medium w-1/4">Env Variable</th>
                            <th className="px-4 py-3 font-medium w-1/4">Config Key</th>
                            <th className="px-4 py-3 font-medium w-1/4">Value</th>
                            <th className="px-4 py-3 font-medium w-1/3">Description</th>
                            <th className="px-4 py-3 font-medium w-1/12 text-center">Category</th>
                            <th className="px-4 py-3 font-medium w-1/12 text-right">Action</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-gray-800">
                        {filteredItems.map(item => (
                            <tr key={item.name} className="hover:bg-gray-800/50 transition-colors group">
                                <td className="px-4 py-3 font-mono text-blue-400">
                                    {item.name}
                                    {item.is_overridden && (
                                        <span className="ml-2 text-[10px] bg-yellow-900/30 text-yellow-500 px-1.5 py-0.5 rounded border border-yellow-800/50">
                                            MODIFIED
                                        </span>
                                    )}
                                </td>
                                <td className="px-4 py-3 font-mono text-gray-400 text-xs">
                                    {item.config_path || '-'}
                                </td>
                                <td className="px-4 py-3">
                                    <span className={clsx(
                                        "font-mono px-1.5 py-0.5 rounded text-xs",
                                        item.is_sensitive ? "text-gray-500 italic" : "bg-gray-800 text-gray-300"
                                    )}>
                                        {item.value || <span className="text-gray-600">null</span>}
                                    </span>
                                </td>
                                <td className="px-4 py-3 text-gray-400 max-w-md truncate" title={item.description}>
                                    <div className="flex items-center gap-2">
                                        <Info className="w-3 h-3 flex-shrink-0" />
                                        <span className="truncate">{item.description}</span>
                                    </div>
                                </td>
                                <td className="px-4 py-3 text-center">
                                    <span className="text-xs bg-gray-800 text-gray-400 px-2 py-1 rounded-full border border-gray-700">
                                        {item.category.replace('EnvCategory::', '')}
                                    </span>
                                </td>
                                <td className="px-4 py-3 text-right">
                                    <button
                                        onClick={() => onEdit(item)}
                                        className="p-1.5 hover:bg-gray-700 rounded-md text-gray-400 hover:text-white transition-colors"
                                        title="Edit Value"
                                    >
                                        <Edit2 className="w-4 h-4" />
                                    </button>
                                </td>
                            </tr>
                        ))}
                        {filteredItems.length === 0 && (
                            <tr>
                                <td colSpan={5} className="px-4 py-8 text-center text-gray-500">
                                    No configuration items found matching "{search}"
                                </td>
                            </tr>
                        )}
                    </tbody>
                </table>
            </div>
        </div>
    );
};
