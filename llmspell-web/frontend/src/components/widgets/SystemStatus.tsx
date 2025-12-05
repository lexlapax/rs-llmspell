import { Activity, Server, Cpu, Database } from 'lucide-react';
import { useSystemStatus } from '../../hooks/useSystemStatus';

export default function SystemStatus() {
    const { status, loading, error } = useSystemStatus();

    const formatUptime = (seconds: number) => {
        const d = Math.floor(seconds / (3600 * 24));
        const h = Math.floor((seconds % (3600 * 24)) / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        return `${d}d ${h}h ${m}m`;
    };

    const stats = [
        {
            name: 'Status',
            value: error ? 'Offline' : (loading ? 'Checking...' : 'Online'),
            icon: Activity,
            color: error ? 'text-red-600' : 'text-green-600',
            bg: error ? 'bg-red-100' : 'bg-green-100'
        },
        {
            name: 'Uptime',
            value: status ? formatUptime(status.uptime_seconds) : '-',
            icon: Server,
            color: 'text-blue-600',
            bg: 'bg-blue-100'
        },
        {
            name: 'Active Sessions',
            value: '-', // Placeholder until sessions API returns count or we calculate it
            icon: Cpu,
            color: 'text-purple-600',
            bg: 'bg-purple-100'
        },
        {
            name: 'Memory Usage',
            value: '-', // Placeholder
            icon: Database,
            color: 'text-orange-600',
            bg: 'bg-orange-100'
        },
    ];

    return (
        <div className="grid grid-cols-1 gap-5 sm:grid-cols-2 lg:grid-cols-4">
            {stats.map((item) => (
                <div key={item.name} className="bg-white overflow-hidden shadow rounded-lg">
                    <div className="p-5">
                        <div className="flex items-center">
                            <div className="flex-shrink-0">
                                <div className={`rounded-md p-3 ${item.bg}`}>
                                    <item.icon className={`h-6 w-6 ${item.color}`} aria-hidden="true" />
                                </div>
                            </div>
                            <div className="ml-5 w-0 flex-1">
                                <dl>
                                    <dt className="text-sm font-medium text-gray-500 truncate">{item.name}</dt>
                                    <dd>
                                        <div className="text-lg font-medium text-gray-900">{item.value}</div>
                                    </dd>
                                </dl>
                            </div>
                        </div>
                    </div>
                </div>
            ))}
        </div>
    );
}
