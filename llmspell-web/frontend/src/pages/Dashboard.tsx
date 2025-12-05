import SystemStatus from '../components/widgets/SystemStatus';
import QuickActions from '../components/widgets/QuickActions';
import RecentActivity from '../components/widgets/RecentActivity';

export default function Dashboard() {
    return (
        <div className="p-6 space-y-6">
            <div className="flex items-center justify-between">
                <h1 className="text-2xl font-bold text-gray-900">Dashboard</h1>
            </div>

            {/* System Status Row */}
            <SystemStatus />

            <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
                {/* Main Content Column (2/3 width on large screens) */}
                <div className="lg:col-span-2 space-y-6">
                    <QuickActions />
                    {/* Placeholder for future charts or main content */}
                    <div className="bg-white shadow rounded-lg p-6 h-64 flex items-center justify-center text-gray-400 border-2 border-dashed border-gray-200">
                        System Performance Chart (Placeholder)
                    </div>
                </div>

                {/* Sidebar Column (1/3 width on large screens) */}
                <div className="lg:col-span-1">
                    <RecentActivity />
                </div>
            </div>
        </div>
    );
}
