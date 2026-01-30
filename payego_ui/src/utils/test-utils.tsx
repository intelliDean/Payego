import { render } from '@testing-library/react';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { AuthProvider } from '../contexts/AuthContext';
import { BrowserRouter } from 'react-router-dom';
import type { ReactElement } from 'react';

/**
 * Custom render function that wraps components with necessary providers
 * Use this instead of @testing-library/react's render for component tests
 */
export function renderWithProviders(ui: ReactElement) {
    const queryClient = new QueryClient({
        defaultOptions: {
            queries: {
                retry: false,
                cacheTime: 0,
            },
            mutations: {
                retry: false,
            },
        },
    });

    return render(
        <BrowserRouter>
            <QueryClientProvider client={queryClient}>
                <AuthProvider>
                    {ui}
                </AuthProvider>
            </QueryClientProvider>
        </BrowserRouter>
    );
}

// Re-export everything from @testing-library/react
export * from '@testing-library/react';

// Override render with our custom version
export { renderWithProviders as render };
