import { Navigate, useLocation } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { type ReactNode } from 'react';

export const ProtectedRoute = ({ children }: { children: ReactNode }) => {
    const { isAuthenticated, devMode } = useAuth();
    const location = useLocation();

    // DEBUG: Log protected route state
    console.log('[ProtectedRoute] devMode:', devMode);
    console.log('[ProtectedRoute] isAuthenticated:', isAuthenticated);
    console.log('[ProtectedRoute] location:', location.pathname);

    // In development mode, backend bypasses authentication
    // Frontend should mirror this behavior to avoid login redirects
    if (devMode) {
        console.log('[ProtectedRoute] ✅ Bypassing auth check - dev mode enabled');
        return <>{children}</>;
    }

    if (!isAuthenticated) {
        // Redirect to login page, but save the current location they were trying to go to
        console.log('[ProtectedRoute] ❌ Redirecting to /login - not authenticated');
        return <Navigate to="/login" state={{ from: location }} replace />;
    }

    console.log('[ProtectedRoute] ✅ Allowing access - authenticated');
    return <>{children}</>;
};
