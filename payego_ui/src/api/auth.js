import client from './client';

export const authApi = {
    login: (email, password) => client.post('/api/auth/login', { email, password }),
    register: (data) => client.post('/api/auth/register', data),
    socialLogin: (idToken, provider) => client.post('/api/auth/social_login', { id_token: idToken, provider }),
    forgotPassword: (email) => client.post('/api/auth/forgot_password', { email }),
    resetPassword: (email, token, newPassword) => client.post('/api/auth/reset_password', { email, token, new_password: newPassword }),
    logout: () => client.post('/api/auth/logout', {}),
    getCurrentUser: () => client.get('/api/user/current'),
};
