import React, { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { GoogleLogin, CredentialResponse } from '@react-oauth/google';
import { EyeIcon, EyeSlashIcon } from '@heroicons/react/24/outline';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import { useAuth } from '../contexts/AuthContext';
import { authApi } from '../api/auth';
import { getErrorMessage } from '../utils/errorHandler';

const loginSchema = z.object({
    email: z.string().email('Please enter a valid email address'),
    password: z.string().min(1, 'Password is required'),
});

type LoginFormValues = z.infer<typeof loginSchema>;

const LoginForm: React.FC = () => {
    const { login } = useAuth();
    const [error, setError] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);
    const [showPassword, setShowPassword] = useState(false);
    const [rememberMe, setRememberMe] = useState(false);

    const navigate = useNavigate();

    const {
        register,
        handleSubmit,
        formState: { errors },
    } = useForm<LoginFormValues>({
        resolver: zodResolver(loginSchema),
    });

    const onSubmit = async (data: LoginFormValues) => {
        setLoading(true);
        setError(null);
        try {
            const response = await authApi.login(data.email, data.password);
            login(response.data.token);
            navigate('/dashboard');
        } catch (err: any) {
            setError(getErrorMessage(err));
        } finally {
            setLoading(false);
        }
    };

    const handleGoogleLogin = async (credentialResponse: CredentialResponse) => {
        if (!credentialResponse.credential) return;
        setLoading(true);
        setError(null);
        try {
            const response = await authApi.socialLogin(credentialResponse.credential, 'google');
            login(response.data.token);
            navigate('/dashboard');
        } catch (err: any) {
            setError(getErrorMessage(err));
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="max-w-md mx-auto mt-8 sm:mt-16 animate-fade-in">
            <div className="card-glass p-8 sm:p-10">
                <div className="text-center mb-8">
                    <div className="relative inline-block mb-6">
                        <div className="absolute inset-0 bg-gradient-to-r from-purple-600 to-blue-600 rounded-2xl blur-lg opacity-50 animate-pulse-slow"></div>
                        <div className="relative w-16 h-16 bg-gradient-to-br from-purple-600 via-blue-600 to-indigo-600 rounded-2xl flex items-center justify-center shadow-2xl">
                            <span className="text-white font-black text-2xl">P</span>
                        </div>
                    </div>
                    <h2 className="text-3xl sm:text-4xl font-black text-gray-900 dark:text-white mb-3">
                        Welcome <span className="gradient-text">Back</span>
                    </h2>
                    <p className="text-gray-600 dark:text-slate-400">Sign in to continue to Payego</p>
                </div>

                <form onSubmit={handleSubmit(onSubmit)} className="space-y-5">
                    <div>
                        <label htmlFor="email" className="input-label">Email Address</label>
                        <div className="input-group">
                            <div className="input-icon">
                                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 12a4 4 0 10-8 0 4 4 0 008 0zm0 0v1.5a2.5 2.5 0 005 0V12a9 9 0 10-9 9m4.5-1.206a8.959 8.959 0 01-4.5 1.207" />
                                </svg>
                            </div>
                            <input
                                id="email"
                                type="email"
                                {...register('email')}
                                className={`input-with-icon ${errors.email ? 'border-red-500' : ''}`}
                                placeholder="you@example.com"
                            />
                        </div>
                        {errors.email && <p className="mt-1 text-xs text-red-500">{errors.email.message}</p>}
                    </div>

                    <div>
                        <label htmlFor="password" className="input-label">Password</label>
                        <div className="input-group">
                            <div className="input-icon">
                                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                                </svg>
                            </div>
                            <input
                                id="password"
                                type={showPassword ? 'text' : 'password'}
                                {...register('password')}
                                className={`input-with-icon pr-12 ${errors.password ? 'border-red-500' : ''}`}
                                placeholder="••••••••"
                            />
                            <button
                                type="button"
                                onClick={() => setShowPassword(!showPassword)}
                                className="absolute right-4 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-purple-600 transition-colors"
                            >
                                {showPassword ? <EyeSlashIcon className="h-5 w-5" /> : <EyeIcon className="h-5 w-5" />}
                            </button>
                        </div>
                        {errors.password && <p className="mt-1 text-xs text-red-500">{errors.password.message}</p>}
                    </div>

                    <div className="flex items-center justify-between">
                        <label className="flex items-center cursor-pointer group">
                            <input
                                type="checkbox"
                                checked={rememberMe}
                                onChange={(e) => setRememberMe(e.target.checked)}
                                className="w-4 h-4 text-purple-600 border-gray-300 rounded focus:ring-purple-500"
                            />
                            <span className="ml-2 text-sm text-gray-700 dark:text-slate-300 group-hover:text-gray-900 dark:group-hover:text-white">Remember me</span>
                        </label>
                        <Link to="/forgot-password" className="text-sm font-semibold gradient-text hover:opacity-80">
                            Forgot password?
                        </Link>
                    </div>

                    <button
                        type="submit"
                        disabled={loading}
                        className="w-full btn-primary-glow btn-lg"
                    >
                        {loading ? 'Signing in...' : 'Sign In'}
                    </button>

                    {error && (
                        <div className="alert-error animate-slide-down">
                            <span className="text-sm font-medium">{error}</span>
                        </div>
                    )}
                </form>

                <div className="relative my-8">
                    <div className="absolute inset-0 flex items-center">
                        <div className="w-full border-t border-gray-300 dark:border-slate-700"></div>
                    </div>
                    <div className="relative flex justify-center text-sm">
                        <span className="px-4 bg-white/80 dark:bg-slate-900/80 text-gray-600 dark:text-slate-400 font-medium rounded-full">Or continue with</span>
                    </div>
                </div>

                <div className="space-y-3">
                    <GoogleLogin
                        onSuccess={handleGoogleLogin}
                        onError={() => setError('Google login failed')}
                        width="100%"
                    />
                </div>

                <p className="mt-8 text-center text-gray-600 dark:text-slate-400">
                    Don't have an account?{' '}
                    <Link to="/register" className="font-bold gradient-text hover:opacity-80">
                        Create Account
                    </Link>
                </p>
            </div>
        </div>
    );
};

export default LoginForm;
