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
