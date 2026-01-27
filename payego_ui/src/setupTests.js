import '@testing-library/jest-dom';
import { vi } from 'vitest';

Object.defineProperty(window, 'localStorage', {
    value: {
        getItem: vi.fn(() => null),
        setItem: vi.fn(() => null),
        removeItem: vi.fn(() => null),
        clear: vi.fn(() => null),
    },
    writable: true,
});

Object.defineProperty(window, 'matchMedia', {
    writable: true,
    value: vi.fn().mockImplementation(query => ({
        matches: false,
        media: query,
        onchange: null,
        addListener: vi.fn(), // deprecated
        removeListener: vi.fn(), // deprecated
        addEventListener: vi.fn(),
        removeEventListener: vi.fn(),
        dispatchEvent: vi.fn(),
    })),
});
