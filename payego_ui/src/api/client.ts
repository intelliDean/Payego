import axios, { AxiosInstance, AxiosResponse } from 'axios';

// Use relative paths in development so Vite proxy can forward to backend
// In production, set VITE_API_URL to the actual backend URL
const API_URL = import.meta.env.VITE_API_URL || '';

const client: AxiosInstance = axios.create({
    baseURL: API_URL,
    headers: {
        'Content-Type': 'application/json',
    },
});

// Add a request interceptor to include the JWT token
client.interceptors.request.use(
    (config) => {
        const token = localStorage.getItem('jwt_token') || sessionStorage.getItem('jwt_token');
        if (token && config.headers) {
            config.headers.Authorization = `Bearer ${token}`;
        }
        return config;
    },
    (error) => Promise.reject(error)
);

// Add a response interceptor to handle errors (e.g., 401 Unauthorized)
client.interceptors.response.use(
    (response: AxiosResponse) => response,
    (error) => {
        if (error.response?.status === 401) {
            // Handle unauthorized error (logout and redirect)
            localStorage.removeItem('jwt_token');
            sessionStorage.removeItem('jwt_token');

            // Only redirect if we are not already on login/register
            if (!window.location.pathname.includes('/login') && !window.location.pathname.includes('/register')) {
                window.location.href = '/login';
            }
        }
        return Promise.reject(error);
    }
);

export default client;
