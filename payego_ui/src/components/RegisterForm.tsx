import React, { useState } from 'react';
import { Link, useNavigate } from 'react-router-dom';
import { EyeIcon, EyeSlashIcon } from '@heroicons/react/24/outline';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import { useAuth } from '../contexts/AuthContext';
import { authApi } from '../api/auth';
import { getErrorMessage } from '../utils/errorHandler';

const registerSchema = z.object({
    email: z.string().email('Please enter a valid email address'),
    username: z.string().min(3, 'Username must be at least 3 characters').optional().or(z.literal('')),
    password: z.string().min(8, 'Password must be at least 8 characters'),
    confirmPassword: z.string().min(1, 'Please confirm your password'),
}).refine((data) => data.password === data.confirmPassword, {
    message: "Passwords don't match",
    path: ["confirmPassword"],
});

type RegisterFormValues = z.infer<typeof registerSchema>;

const RegisterForm: React.FC = () => {
    const { login } = useAuth();
    const [error, setError] = useState<string | null>(null);
    const [successMsg, setSuccessMsg] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);
    const [showPassword, setShowPassword] = useState(false);
    const [showConfirmPassword, setShowConfirmPassword] = useState(false);
    const [showVerification, setShowVerification] = useState(false);

    const navigate = useNavigate();

    const {
        register,
        handleSubmit,
        watch,
        formState: { errors },
    } = useForm<RegisterFormValues>({
        resolver: zodResolver(registerSchema),
    });

    const password = watch('password', '');

    const evaluatePasswordStrength = (pass: string) => {
        let score = 0;
        if (pass.length >= 8) score++;
        if (/[A-Z]/.test(pass)) score++;
        if (/[a-z]/.test(pass)) score++;
        if (/[0-9]/.test(pass)) score++;
        if (/[^A-Za-z0-9]/.test(pass)) score++;

        if (score <= 2) return { label: 'Weak', color: 'bg-red-500' };
        if (score <= 3) return { label: 'Fair', color: 'bg-yellow-500' };
        if (score <= 4) return { label: 'Good', color: 'bg-blue-500' };
        return { label: 'Strong', color: 'bg-green-500' };
    };

    const strength = evaluatePasswordStrength(password);

    const onSubmit = async (data: RegisterFormValues) => {
        setLoading(true);
        setError(null);
        try {
            const response = await authApi.register({
                email: data.email,
                password: data.password,
                username: data.username || undefined,
            });
            login(response.data.token);
            setShowVerification(true);
        } catch (err: any) {
            setError(getErrorMessage(err));
            setLoading(false);
        }
    };

    const handleResend = async () => {
        try {
            await authApi.resendVerification();
            setSuccessMsg("Verification email resent!");
        } catch (err: any) {
            setError(getErrorMessage(err));
        }
    };

    if (showVerification) {
        return (
            <div className="max-w-md mx-auto mt-8 sm:mt-16 animate-fade-in">
                <div className="card-glass p-8 sm:p-10 text-center">
                    <div className="w-20 h-20 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center mx-auto mb-6 shadow-glow-green">
                        <svg className="w-10 h-10 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M3 8l7.89 5.26a2 2 0 002.22 0L21 8M5 19h14a2 2 0 002-2V7a2 2 0 00-2-2H5a2 2 0 00-2 2v10a2 2 0 002 2z" />
                        </svg>
                    </div>
                    <h2 className="text-3xl font-black text-gray-900 dark:text-white mb-4">Check Your <span className="gradient-text">Email</span></h2>
                    <p className="text-gray-600 dark:text-slate-400 mb-8">
                        We've sent a verification link to your email address. Please click the link to secure your account.
                    </p>

                    {successMsg && <div className="mb-4 text-green-500 text-sm font-bold">{successMsg}</div>}
                    {error && <div className="mb-4 text-red-500 text-sm font-bold">{error}</div>}

                    <div className="space-y-4">
                        <button onClick={() => navigate('/dashboard')} className="w-full btn-primary-glow btn-lg">
                            Go to Dashboard
                        </button>
                        <button onClick={handleResend} className="w-full text-slate-500 hover:text-white transition-colors text-sm font-medium">
                            Didn't receive an email? Resend
                        </button>
                    </div>
                </div>
            </div>
        );
    }

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
                        Create <span className="gradient-text">Account</span>
                    </h2>
                    <p className="text-gray-600 dark:text-slate-400">Join Payego and start managing your finances</p>
                </div>

                <form onSubmit={handleSubmit(onSubmit)} className="space-y-5">
                    <div>
                        <label className="input-label">Email Address</label>
                        <input
                            type="email"
                            {...register('email')}
                            className={`input-with-icon ${errors.email ? 'border-red-500' : ''}`}
                            placeholder="you@example.com"
                        />
                        {errors.email && <p className="mt-1 text-xs text-red-500">{errors.email.message}</p>}
                    </div>

                    <div>
                        <label className="input-label">Username (optional)</label>
                        <input
                            type="text"
                            {...register('username')}
                            className={`input-with-icon ${errors.username ? 'border-red-500' : ''}`}
                            placeholder="johndoe"
                        />
                        {errors.username && <p className="mt-1 text-xs text-red-500">{errors.username.message}</p>}
                    </div>

                    <div>
                        <label className="input-label">Password</label>
                        <div className="input-group">
                            <input
                                type={showPassword ? 'text' : 'password'}
                                {...register('password')}
                                className={`input-with-icon pr-12 ${errors.password ? 'border-red-500' : ''}`}
                                placeholder="••••••••"
                            />
                            <button
                                type="button"
                                onClick={() => setShowPassword(!showPassword)}
                                className="absolute right-4 top-1/2 transform -translate-y-1/2 text-gray-400"
                            >
                                {showPassword ? <EyeSlashIcon className="h-5 w-5" /> : <EyeIcon className="h-5 w-5" />}
                            </button>
                        </div>
                        {password && (
                            <div className="mt-2 text-xs">
                                <div className="w-full bg-gray-200 h-1 rounded-full overflow-hidden">
                                    <div className={`h-full transition-all ${strength.color}`} style={{ width: strength ? '100%' : '0%' }}></div>
                                </div>
                                <span className="mt-1 inline-block">Strength: {strength.label}</span>
                            </div>
                        )}
                        {errors.password && <p className="mt-1 text-xs text-red-500">{errors.password.message}</p>}
                    </div>

                    <div>
                        <label className="input-label">Confirm Password</label>
                        <div className="input-group">
                            <input
                                type={showConfirmPassword ? 'text' : 'password'}
                                {...register('confirmPassword')}
                                className={`input-with-icon pr-12 ${errors.confirmPassword ? 'border-red-500' : ''}`}
                                placeholder="••••••••"
                            />
                            <button
                                type="button"
                                onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                                className="absolute right-4 top-1/2 transform -translate-y-1/2 text-gray-400"
                            >
                                {showConfirmPassword ? <EyeSlashIcon className="h-5 w-5" /> : <EyeIcon className="h-5 w-5" />}
                            </button>
                        </div>
                        {errors.confirmPassword && <p className="mt-1 text-xs text-red-500">{errors.confirmPassword.message}</p>}
                    </div>

                    <button type="submit" disabled={loading} className="w-full btn-primary-glow btn-lg">
                        {loading ? 'Creating Account...' : 'Create Account'}
                    </button>

                    {error && <div className="alert-error"><span className="text-sm">{error}</span></div>}
                </form>

                <p className="mt-8 text-center text-gray-600 dark:text-slate-400">
                    Already have an account?{' '}
                    <Link to="/login" className="font-bold gradient-text hover:opacity-80">
                        Sign In
                    </Link>
                </p>
            </div>
        </div>
    );
};

export default RegisterForm;
