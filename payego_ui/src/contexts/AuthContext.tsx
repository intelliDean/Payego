import React, { createContext, useContext, useState, useEffect, ReactNode } from 'react';
import { User } from '../types';
import { authApi } from '../api/auth';

interface AuthContextType {
    user: User | null;
    isAuthenticated: boolean;
    isLoading: boolean;
    login: (token: string) => void;
    logout: () => void;
    refreshUser: () => Promise<void>;
}

const AuthContext = createContext<AuthContextType | undefined>(undefined);

export const AuthProvider: React.FC<{ children: ReactNode }> = ({ children }) => {
    const [user, setUser] = useState<User | null>(null);
    const [isAuthenticated, setIsAuthenticated] = useState<boolean>(!!localStorage.getItem('jwt_token'));
    const [isLoading, setIsLoading] = useState<boolean>(true);

    const refreshUser = async () => {
        const token = localStorage.getItem('jwt_token');
        if (!token) {
            setUser(null);
            setIsAuthenticated(false);
            setIsLoading(false);
            return;
        }

        try {
            const userData = await authApi.getCurrentUser();
            setUser(userData);
            setIsAuthenticated(true);
        } catch (error) {
            console.error('Failed to fetch user:', error);
            localStorage.removeItem('jwt_token');
            setUser(null);
            setIsAuthenticated(false);
        } finally {
            setIsLoading(false);
        }
    };

    useEffect(() => {
        refreshUser();
    }, []);

    const login = (token: string) => {
        localStorage.setItem('jwt_token', token);
        setIsAuthenticated(true);
        refreshUser();
    };

    const logout = () => {
        localStorage.removeItem('jwt_token');
        setUser(null);
        setIsAuthenticated(false);
    };

    return (
        <AuthContext.Provider value={{ user, isAuthenticated, isLoading, login, logout, refreshUser }}>
            {children}
        </AuthContext.Provider>
    );
};

export const useAuth = () => {
    const context = useContext(AuthContext);
    if (context === undefined) {
        throw new Error('useAuth must be used within an AuthProvider');
    }
    return context;
};
