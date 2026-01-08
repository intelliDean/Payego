import client from './client';

export const authApi = {
    login: (email, password) => client.post('/api/login', { email, password }),
    register: (data) => client.post('/api/register', data),
    socialLogin: (idToken, provider) => client.post('/api/social_login', { id_token: idToken, provider }),
    forgotPassword: (email) => client.post('/api/forgot_password', { email }),
    resetPassword: (email, token, newPassword) => client.post('/api/reset_password', { email, token, new_password: newPassword }),
    logout: () => client.post('/api/logout', {}),
    getCurrentUser: () => client.get('/api/current_user'),
};
