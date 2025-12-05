import { Clock, CheckCircle, XCircle, PlayCircle } from 'lucide-react';
import { useRecentActivity } from '../../hooks/useRecentActivity';

export default function RecentActivity() {
    const { sessions, loading, error } = useRecentActivity();

    const getStatusIcon = (status: string) => {
        switch (status) {
            case 'completed': return <CheckCircle className="h-5 w-5 text-green-500" />;
            case 'active': return <PlayCircle className="h-5 w-5 text-blue-500" />;
            case 'failed': return <XCircle className="h-5 w-5 text-red-500" />;
            default: return <Clock className="h-5 w-5 text-gray-500" />;
        }
    };

    const formatTime = (timestamp: number) => {
        return new Date(timestamp * 1000).toLocaleString();
    };

    if (loading) return <div className="p-4 text-center text-gray-500">Loading activity...</div>;
    if (error) return <div className="p-4 text-center text-red-500">Failed to load activity</div>;

    return (
        <div className="bg-white shadow rounded-lg">
            <div className="px-4 py-5 sm:px-6 border-b border-gray-200">
                <h3 className="text-lg leading-6 font-medium text-gray-900">Recent Activity</h3>
            </div>
            <ul className="divide-y divide-gray-200">
                {sessions.length === 0 ? (
                    <li className="px-4 py-4 text-center text-gray-500">No recent activity</li>
                ) : (
                    sessions.map((item) => (
                        <li key={item.id} className="px-4 py-4 sm:px-6 hover:bg-gray-50">
                            <div className="flex items-center justify-between">
                                <div className="flex items-center">
                                    <div className="flex-shrink-0 mr-4">
                                        {getStatusIcon(item.status)}
                                    </div>
                                    <div>
                                        <p className="text-sm font-medium text-blue-600 truncate">{item.id.substring(0, 8)}...</p>
                                        <p className="text-sm text-gray-500">Session</p>
                                    </div>
                                </div>
                                <div className="flex-shrink-0">
                                    <p className="text-sm text-gray-500">{formatTime(item.created_at)}</p>
                                </div>
                            </div>
                        </li>
                    ))
                )}
            </ul>
        </div>
    );
}
