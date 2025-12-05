import { useState } from 'react';
import { Link, useLocation } from 'react-router-dom';
import {
    LayoutDashboard,
    MessageSquare,
    Brain,
    Bot,
    Wrench,
    Settings,
    Menu,
    X
} from 'lucide-react';
import clsx from 'clsx';

interface LayoutProps {
    children: React.ReactNode;
}

export default function Layout({ children }: LayoutProps) {
    const [isSidebarOpen, setIsSidebarOpen] = useState(false);
    const location = useLocation();

    const navigation = [
        { name: 'Dashboard', href: '/', icon: LayoutDashboard },
        { name: 'Sessions', href: '/sessions', icon: MessageSquare },
        { name: 'Memory', href: '/memory', icon: Brain },
        { name: 'Agents', href: '/agents', icon: Bot },
        { name: 'Tools', href: '/tools', icon: Wrench },
    ];

    return (
        <div className="min-h-screen bg-gray-50 flex">
            {/* Mobile sidebar backdrop */}
            {isSidebarOpen && (
                <div
                    className="fixed inset-0 z-20 bg-black/50 lg:hidden"
                    onClick={() => setIsSidebarOpen(false)}
                />
            )}

            {/* Sidebar */}
            <div className={clsx(
                "fixed inset-y-0 left-0 z-30 w-64 bg-white border-r border-gray-200 transform transition-transform duration-200 ease-in-out lg:translate-x-0 lg:static lg:inset-0",
                isSidebarOpen ? "translate-x-0" : "-translate-x-full"
            )}>
                <div className="flex items-center justify-between h-16 px-6 border-b border-gray-200">
                    <span className="text-xl font-bold text-gray-900">rs-llmspell</span>
                    <button
                        onClick={() => setIsSidebarOpen(false)}
                        className="lg:hidden p-1 rounded-md hover:bg-gray-100"
                    >
                        <X className="w-6 h-6 text-gray-500" />
                    </button>
                </div>

                <nav className="p-4 space-y-1">
                    {navigation.map((item) => {
                        const isActive = location.pathname === item.href;
                        return (
                            <Link
                                key={item.name}
                                to={item.href}
                                className={clsx(
                                    "flex items-center px-4 py-2 text-sm font-medium rounded-md transition-colors",
                                    isActive
                                        ? "bg-blue-50 text-blue-700"
                                        : "text-gray-700 hover:bg-gray-50 hover:text-gray-900"
                                )}
                            >
                                <item.icon className={clsx(
                                    "w-5 h-5 mr-3",
                                    isActive ? "text-blue-700" : "text-gray-400 group-hover:text-gray-500"
                                )} />
                                {item.name}
                            </Link>
                        );
                    })}
                </nav>

                <div className="absolute bottom-0 w-full p-4 border-t border-gray-200">
                    <Link
                        to="/settings"
                        className="flex items-center px-4 py-2 text-sm font-medium text-gray-700 rounded-md hover:bg-gray-50 hover:text-gray-900"
                    >
                        <Settings className="w-5 h-5 mr-3 text-gray-400" />
                        Settings
                    </Link>
                </div>
            </div>

            {/* Main Content */}
            <div className="flex-1 flex flex-col min-w-0 overflow-hidden">
                {/* Header */}
                <header className="bg-white border-b border-gray-200 h-16 flex items-center justify-between px-4 sm:px-6 lg:px-8">
                    <button
                        onClick={() => setIsSidebarOpen(true)}
                        className="lg:hidden p-2 rounded-md text-gray-400 hover:text-gray-500 hover:bg-gray-100"
                    >
                        <Menu className="w-6 h-6" />
                    </button>

                    <div className="flex items-center">
                        <div className="flex items-center space-x-2">
                            <div className="w-2 h-2 rounded-full bg-green-500"></div>
                            <span className="text-sm font-medium text-gray-600">System Online</span>
                        </div>
                    </div>
                </header>

                {/* Page Content */}
                <main className="flex-1 overflow-auto">
                    {children}
                </main>
            </div>
        </div>
    );
}
