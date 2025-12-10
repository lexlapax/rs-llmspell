import { Navigate, useLocation } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import { type ReactNode } from 'react';

export const ProtectedRoute = ({ children }: { children: ReactNode }) => {
    const { isAuthenticated } = useAuth();
    const location = useLocation();

    // Check for dev mode bypass in local storage if we want to handle mixed mode in UI?
    // Actually, backend handles bypass, but frontend needs to know if it should enforce login locally.
    // If dev mode is on, backend returns success even without token.
    // But we want to enforce UI flow.
    // Let's assume production mode enforcement.

    if (!isAuthenticated) {
        // Redirect to login page, but save the current location they were trying to go to
        return <Navigate to="/login" state={{ from: location }} replace />;
    }

    return <>{children}</>;
};
