import { Navigate, useLocation } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { type ReactNode } from 'react';

export const ProtectedRoute = ({ children }: { children: ReactNode }) => {
    const { isAuthenticated, devMode } = useAuth();
    const location = useLocation();

    // In development mode, backend bypasses authentication
    // Frontend should mirror this behavior to avoid login redirects
    if (devMode) {
        return <>{children}</>;
    }

    if (!isAuthenticated) {
        // Redirect to login page, but save the current location they were trying to go to
        return <Navigate to="/login" state={{ from: location }} replace />;
    }

    return <>{children}</>;
};
