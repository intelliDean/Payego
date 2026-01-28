import client from './client';
import { User } from '@/types';

export const authApi = {
    login: (email: string, password: string) => client.post('/api/auth/login', { email, password }),
    register: (data: any) => client.post('/api/auth/register', data),
    socialLogin: (idToken: string, provider: string) => client.post('/api/auth/social_login', { id_token: idToken, provider }),
    forgotPassword: (email: string) => client.post('/api/auth/forgot_password', { email }),
    resetPassword: (email: string, token: string, newPassword: string) => client.post('/api/auth/reset_password', { email, token, new_password: newPassword }),
    logout: () => client.post('/api/auth/logout', {}),
    getCurrentUser: () => client.get<User>('/api/user/current').then(res => res.data),
    verifyEmail: (token: string) => client.get(`/api/auth/verify-email?token=${token}`),
    resendVerification: () => client.post('/api/auth/resend-verification', {}),
};
