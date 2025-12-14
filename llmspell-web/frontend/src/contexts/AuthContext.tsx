import { createContext, useContext, useState, useEffect, type ReactNode } from 'react';

interface AuthContextType {
    isAuthenticated: boolean;
    token: string | null;
    login: (token: string) => void;
    logout: () => void;
    devMode: boolean;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider = ({ children }: { children: ReactNode }) => {
    const [token, setToken] = useState<string | null>(localStorage.getItem('token'));
    const [devMode, setDevMode] = useState(false);
    const [devModeChecked, setDevModeChecked] = useState(false);

    // Check backend dev_mode on mount
    useEffect(() => {
        const checkBackendDevMode = async () => {
            try {
                // For dev server (npm run dev), check Vite environment first
                const viteDevMode = import.meta.env.MODE === 'development';
                if (viteDevMode) {
                    console.log('[AuthContext] Vite dev mode detected');
                    setDevMode(true);
                    setDevModeChecked(true);
                    return;
                }

                // For embedded UI (production build), check backend
                console.log('[AuthContext] Checking backend dev_mode via /health');
                const response = await fetch('/health');
                if (!response.ok) {
                    throw new Error(`Health check failed: ${response.status}`);
                }
                const data = await response.json();
                const backendDevMode = data.dev_mode === true;
                console.log('[AuthContext] Backend dev_mode:', backendDevMode);
                setDevMode(backendDevMode);
            } catch (error) {
                console.error('[AuthContext] Failed to check dev mode:', error);
                setDevMode(false); // Default to production mode on error
            } finally {
                setDevModeChecked(true);
            }
        };

        checkBackendDevMode();
    }, []);

    useEffect(() => {
        // Sync state if localStorage changes (e.g. other tabs)
        const handleStorageChange = () => {
            setToken(localStorage.getItem('token'));
        };
        window.addEventListener('storage', handleStorageChange);
        return () => window.removeEventListener('storage', handleStorageChange);
    }, []);

    const login = (newToken: string) => {
        localStorage.setItem('token', newToken);
        setToken(newToken);
    };

    const logout = () => {
        localStorage.removeItem('token');
        setToken(null);
    };

    // Don't render until dev mode is checked (prevents flash of login screen)
    if (!devModeChecked) {
        return (
            <div style={{
                display: 'flex',
                justifyContent: 'center',
                alignItems: 'center',
                height: '100vh',
                fontSize: '14px',
                color: '#666'
            }}>
                Loading...
            </div>
        );
    }

    const value = {
        isAuthenticated: devMode || !!token,
        token,
        login,
        logout,
        devMode
    };

    return (
        <AuthContext.Provider value={value}>
            {children}
        </AuthContext.Provider>
    );
};

// eslint-disable-next-line react-refresh/only-export-components
export const useAuth = () => {
    const context = useContext(AuthContext);
    if (context === undefined) {
        throw new Error('useAuth must be used within an AuthProvider');
    }
    return context;
};
