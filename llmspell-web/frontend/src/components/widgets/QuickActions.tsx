import { Link } from 'react-router-dom';
import { Plus, Search, Play } from 'lucide-react';

export default function QuickActions() {
    const actions = [
        { name: 'New Session', href: '/sessions', icon: Plus, color: 'bg-blue-600 hover:bg-blue-700' },
        { name: 'Search Memory', href: '/memory', icon: Search, color: 'bg-purple-600 hover:bg-purple-700' },
        { name: 'Run Agent', href: '/agents', icon: Play, color: 'bg-green-600 hover:bg-green-700' },
    ];

    return (
        <div className="bg-white shadow rounded-lg p-6">
            <h3 className="text-lg leading-6 font-medium text-gray-900 mb-4">Quick Actions</h3>
            <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
                {actions.map((action) => (
                    <Link
                        key={action.name}
                        to={action.href}
                        className={`relative flex items-center justify-center px-4 py-3 border border-transparent text-sm font-medium rounded-md text-white shadow-sm ${action.color} focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500`}
                    >
                        <action.icon className="mr-2 h-5 w-5" aria-hidden="true" />
                        <span>{action.name}</span>
                    </Link>
                ))}
            </div>
        </div>
    );
}
