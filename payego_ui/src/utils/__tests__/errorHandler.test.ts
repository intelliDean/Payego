import { describe, it, expect } from 'vitest';
import { getErrorMessage, isNetworkError, isAuthError, isValidationError } from '../errorHandler';

describe('getErrorMessage', () => {
    describe('API error responses', () => {
        it('extracts message from response.data.message', () => {
            const error = {
                response: {
                    data: { message: 'Invalid credentials' },
                    status: 401,
                },
            };
            expect(getErrorMessage(error)).toBe('Invalid credentials');
        });

        it('extracts message from response.data.error', () => {
            const error = {
                response: {
                    data: { error: 'User not found' },
                    status: 404,
                },
            };
            expect(getErrorMessage(error)).toBe('User not found');
        });

        it('handles validation error arrays with message objects', () => {
            const error = {
                response: {
                    data: {
                        errors: [
                            { message: 'Email is required', field: 'email' },
                            { message: 'Password too short', field: 'password' },
                        ],
                    },
                    status: 422,
                },
            };
            expect(getErrorMessage(error)).toBe('Email is required, Password too short');
        });

        it('handles validation error arrays with strings', () => {
            const error = {
                response: {
                    data: {
                        errors: ['Email is required', 'Password too short'],
                    },
                    status: 422,
                },
            };
            expect(getErrorMessage(error)).toBe('Email is required, Password too short');
        });

        it('handles mixed validation error arrays', () => {
            const error = {
                response: {
                    data: {
                        errors: [
                            { message: 'Email is required' },
                            'Password too short',
                            { field: 'username' }, // No message
                        ],
                    },
                    status: 422,
                },
            };
            expect(getErrorMessage(error)).toBe('Email is required, Password too short, Validation error');
        });
    });

    describe('HTTP status code fallbacks', () => {
        it('returns appropriate message for 400 Bad Request', () => {
            const error = {
                response: {
                    data: {},
                    status: 400,
                },
            };
            expect(getErrorMessage(error)).toBe('Invalid request. Please check your input.');
        });

        it('returns appropriate message for 401 Unauthorized', () => {
            const error = {
                response: {
                    data: {},
                    status: 401,
                },
            };
            expect(getErrorMessage(error)).toBe('Session expired. Please log in again.');
        });

        it('returns appropriate message for 403 Forbidden', () => {
            const error = {
                response: {
                    data: {},
                    status: 403,
                },
            };
            expect(getErrorMessage(error)).toBe("You don't have permission to perform this action.");
        });

        it('returns appropriate message for 404 Not Found', () => {
            const error = {
                response: {
                    data: {},
                    status: 404,
                },
            };
            expect(getErrorMessage(error)).toBe('Resource not found.');
        });

        it('returns appropriate message for 409 Conflict', () => {
            const error = {
                response: {
                    data: {},
                    status: 409,
                },
            };
            expect(getErrorMessage(error)).toBe('This action conflicts with existing data.');
        });

        it('returns appropriate message for 422 Unprocessable Entity', () => {
            const error = {
                response: {
                    data: {},
                    status: 422,
                },
            };
            expect(getErrorMessage(error)).toBe('Validation failed. Please check your input.');
        });

        it('returns appropriate message for 429 Too Many Requests', () => {
            const error = {
                response: {
                    data: {},
                    status: 429,
                },
            };
            expect(getErrorMessage(error)).toBe('Too many requests. Please try again later.');
        });

        it('returns appropriate message for 500 Internal Server Error', () => {
            const error = {
                response: {
                    data: {},
                    status: 500,
                },
            };
            expect(getErrorMessage(error)).toBe('Server error. Please try again later.');
        });

        it('returns appropriate message for 502 Bad Gateway', () => {
            const error = {
                response: {
                    data: {},
                    status: 502,
                },
            };
            expect(getErrorMessage(error)).toBe('Server error. Please try again later.');
        });

        it('returns appropriate message for 503 Service Unavailable', () => {
            const error = {
                response: {
                    data: {},
                    status: 503,
                },
            };
            expect(getErrorMessage(error)).toBe('Server error. Please try again later.');
        });

        it('returns statusText for unknown status codes', () => {
            const error = {
                response: {
                    data: {},
                    status: 418,
                    statusText: "I'm a teapot",
                },
            };
            expect(getErrorMessage(error)).toBe("I'm a teapot");
        });

        it('returns generic message when statusText is missing', () => {
            const error = {
                response: {
                    data: {},
                    status: 418,
                },
            };
            expect(getErrorMessage(error)).toBe('Request failed');
        });
    });

    describe('network errors', () => {
        it('handles network errors (no response)', () => {
            const error = {
                request: {},
            };
            expect(getErrorMessage(error)).toBe('Network error. Please check your internet connection.');
        });
    });

    describe('other errors', () => {
        it('extracts message from error.message', () => {
            const error = {
                message: 'Something went wrong',
            };
            expect(getErrorMessage(error)).toBe('Something went wrong');
        });

        it('returns generic message for unknown errors', () => {
            const error = {};
            expect(getErrorMessage(error)).toBe('An unexpected error occurred. Please try again.');
        });

        it('handles null/undefined errors', () => {
            expect(getErrorMessage(null)).toBe('An unexpected error occurred. Please try again.');
            expect(getErrorMessage(undefined)).toBe('An unexpected error occurred. Please try again.');
        });
    });
});

describe('isNetworkError', () => {
    it('returns true for network errors', () => {
        const error = {
            request: {},
        };
        expect(isNetworkError(error)).toBe(true);
    });

    it('returns false for API errors with response', () => {
        const error = {
            request: {},
            response: {
                status: 500,
            },
        };
        expect(isNetworkError(error)).toBe(false);
    });

    it('returns false for other errors', () => {
        const error = {
            message: 'Something went wrong',
        };
        expect(isNetworkError(error)).toBe(false);
    });
});

describe('isAuthError', () => {
    it('returns true for 401 errors', () => {
        const error = {
            response: {
                status: 401,
            },
        };
        expect(isAuthError(error)).toBe(true);
    });

    it('returns false for other status codes', () => {
        const error = {
            response: {
                status: 400,
            },
        };
        expect(isAuthError(error)).toBe(false);
    });

    it('returns false for network errors', () => {
        const error = {
            request: {},
        };
        expect(isAuthError(error)).toBe(false);
    });
});

describe('isValidationError', () => {
    it('returns true for 400 errors', () => {
        const error = {
            response: {
                status: 400,
            },
        };
        expect(isValidationError(error)).toBe(true);
    });

    it('returns true for 422 errors', () => {
        const error = {
            response: {
                status: 422,
            },
        };
        expect(isValidationError(error)).toBe(true);
    });

    it('returns false for other status codes', () => {
        const error = {
            response: {
                status: 401,
            },
        };
        expect(isValidationError(error)).toBe(false);
    });

    it('returns false for network errors', () => {
        const error = {
            request: {},
        };
        expect(isValidationError(error)).toBe(false);
    });
});
