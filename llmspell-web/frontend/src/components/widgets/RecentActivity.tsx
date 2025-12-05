import { Clock, CheckCircle, XCircle, PlayCircle } from 'lucide-react';

export default function RecentActivity() {
    // Mock data
    const activity = [
        { id: 1, type: 'Session', name: 'Research Assistant', status: 'completed', time: '2 mins ago' },
        { id: 2, type: 'Agent', name: 'Code Reviewer', status: 'running', time: '15 mins ago' },
        { id: 3, type: 'Tool', name: 'Web Search', status: 'failed', time: '1 hour ago' },
        { id: 4, type: 'Session', name: 'Creative Writing', status: 'completed', time: '3 hours ago' },
    ];

    const getStatusIcon = (status: string) => {
        switch (status) {
            case 'completed': return <CheckCircle className="h-5 w-5 text-green-500" />;
            case 'running': return <PlayCircle className="h-5 w-5 text-blue-500" />;
            case 'failed': return <XCircle className="h-5 w-5 text-red-500" />;
            default: return <Clock className="h-5 w-5 text-gray-500" />;
        }
    };

    return (
        <div className="bg-white shadow rounded-lg">
            <div className="px-4 py-5 sm:px-6 border-b border-gray-200">
                <h3 className="text-lg leading-6 font-medium text-gray-900">Recent Activity</h3>
            </div>
            <ul className="divide-y divide-gray-200">
                {activity.map((item) => (
                    <li key={item.id} className="px-4 py-4 sm:px-6 hover:bg-gray-50">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center">
                                <div className="flex-shrink-0 mr-4">
                                    {getStatusIcon(item.status)}
                                </div>
                                <div>
                                    <p className="text-sm font-medium text-blue-600 truncate">{item.name}</p>
                                    <p className="text-sm text-gray-500">{item.type}</p>
                                </div>
                            </div>
                            <div className="flex-shrink-0">
                                <p className="text-sm text-gray-500">{item.time}</p>
                            </div>
                        </div>
                    </li>
                ))}
            </ul>
        </div>
    );
}
