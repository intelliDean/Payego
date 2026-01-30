/**
 * Error handling utility for extracting user-friendly error messages from API responses
 */

export interface ApiErrorResponse {
    message?: string;
    error?: string;
    errors?: Array<{ message?: string; field?: string } | string>;
}

/**
 * Extracts a user-friendly error message from an error object
 * @param error - The error object (typically from Axios)
 * @returns A user-friendly error message
 */
export function getErrorMessage(error: any): string {
    // Handle null/undefined errors
    if (!error) {
        return 'An unexpected error occurred. Please try again.';
    }

    // Handle Axios errors with response
    if (error.response) {
        const data: ApiErrorResponse = error.response.data;

        // Check for message field (most common)
        if (data?.message) {
            return data.message;
        }

        // Check for error field
        if (data?.error) {
            return data.error;
        }

        // Check for errors array (validation errors)
        if (data?.errors && Array.isArray(data.errors)) {
            const messages = data.errors.map(e => {
                if (typeof e === 'string') return e;
                if (e.message) return e.message;
                return 'Validation error';
            });
            return messages.join(', ');
        }

        // Fallback to status-based messages
        const status = error.response.status;
        switch (status) {
            case 400:
                return 'Invalid request. Please check your input.';
            case 401:
                return 'Session expired. Please log in again.';
            case 403:
                return 'You don\'t have permission to perform this action.';
            case 404:
                return 'Resource not found.';
            case 409:
                return 'This action conflicts with existing data.';
            case 422:
                return 'Validation failed. Please check your input.';
            case 429:
                return 'Too many requests. Please try again later.';
            case 500:
            case 502:
            case 503:
                return 'Server error. Please try again later.';
            default:
                return error.response.statusText || 'Request failed';
        }
    }

    // Handle network errors (no response received)
    if (error.request) {
        return 'Network error. Please check your internet connection.';
    }

    // Handle other errors
    if (error.message) {
        return error.message;
    }

    return 'An unexpected error occurred. Please try again.';
}

/**
 * Checks if an error is a network error
 */
export function isNetworkError(error: any): boolean {
    return !!error && !!error.request && !error.response;
}

/**
 * Checks if an error is an authentication error
 */
export function isAuthError(error: any): boolean {
    return error.response?.status === 401;
}

/**
 * Checks if an error is a validation error
 */
export function isValidationError(error: any): boolean {
    return error.response?.status === 400 || error.response?.status === 422;
}
