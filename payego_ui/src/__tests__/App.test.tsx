import { render } from '@testing-library/react';
import { describe, it, expect } from 'vitest';
import App from '../App';
import { AuthProvider } from '../contexts/AuthContext';
import { QueryProvider } from '../providers/QueryProvider';

describe('App', () => {
    it('renders without crashing', () => {
        render(
            <QueryProvider>
                <AuthProvider>
                    <App />
                </AuthProvider>
            </QueryProvider>
        );
        expect(true).toBe(true);
    });
});
