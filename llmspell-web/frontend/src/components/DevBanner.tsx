import { AlertTriangle } from 'lucide-react';
import { useAuth } from '../contexts/AuthContext';

export const DevBanner = () => {
    const { devMode } = useAuth();

    if (!devMode) return null;

    return (
        <div className="dev-mode-banner">
            <AlertTriangle className="w-4 h-4" />
            <span>Development Mode - Authentication Disabled</span>
        </div>
    );
};
