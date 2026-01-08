import axios from 'axios';

const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8080';

const client = axios.create({
    baseURL: API_URL,
    headers: {
        'Content-Type': 'application/json',
    },
});

// Add a request interceptor to include the JWT token
client.interceptors.request.use(
    (config) => {
        const token = localStorage.getItem('jwt_token') || sessionStorage.getItem('jwt_token');
        if (token) {
            config.headers.Authorization = `Bearer ${token}`;
        }
        return config;
    },
    (error) => Promise.reject(error)
);

// Add a response interceptor to handle errors (e.g., 401 Unauthorized)
client.interceptors.response.use(
    (response) => response,
    (error) => {
        if (error.response?.status === 401) {
            // Handle unauthorized error (logout and redirect)
            localStorage.removeItem('jwt_token');
            sessionStorage.removeItem('jwt_token');
            // We can't use navigate here since it's not a component, 
            // but we can dispatch a custom event or use window.location
            window.location.href = '/login';
        }
        return Promise.reject(error);
    }
);

export default client;
