import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { GoogleOAuthProvider } from '@react-oauth/google';
import { QueryProvider } from './providers/QueryProvider';
import { AuthProvider } from './contexts/AuthContext';
import './index.css';
import App from './App';

createRoot(document.getElementById('root')!).render(
    <StrictMode>
        <GoogleOAuthProvider clientId={import.meta.env.VITE_GOOGLE_CLIENT_ID || ""}>
            <QueryProvider>
                <AuthProvider>
                    <App />
                </AuthProvider>
            </QueryProvider>
        </GoogleOAuthProvider>
    </StrictMode>
);
