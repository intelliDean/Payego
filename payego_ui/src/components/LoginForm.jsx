import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { Link, useNavigate, useLocation } from 'react-router-dom';
import { GoogleLogin } from '@react-oauth/google';
import { EyeIcon, EyeSlashIcon } from '@heroicons/react/24/outline';

function LoginForm({ setAuth }) {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [newPassword, setNewPassword] = useState('');
    const [confirmNewPassword, setConfirmNewPassword] = useState('');
    const [error, setError] = useState(null);
    const [success, setSuccess] = useState(null);
    const [loading, setLoading] = useState(false);
    const [showPassword, setShowPassword] = useState(false);
    const [showNewPassword, setShowNewPassword] = useState(false);
    const [showConfirmNewPassword, setShowConfirmNewPassword] = useState(false);
    const [rememberMe, setRememberMe] = useState(false);
    const [showForgotPassword, setShowForgotPassword] = useState(false);
    const [resetToken, setResetToken] = useState('');
    const navigate = useNavigate();
    const location = useLocation();

    // Check for reset password query parameters
    useEffect(() => {
        const params = new URLSearchParams(location.search);
        const emailParam = params.get('email');
        const tokenParam = params.get('token');
        if (emailParam && tokenParam) {
            setEmail(emailParam);
            setResetToken(tokenParam);
            setShowForgotPassword(true);
        }
    }, [location]);

    // Toggle password visibility
    const togglePasswordVisibility = () => setShowPassword(!showPassword);
    const toggleNewPasswordVisibility = () => setShowNewPassword(!showNewPassword);
    const toggleConfirmNewPasswordVisibility = () => setShowConfirmNewPassword(!showConfirmNewPassword);

    const handleLogin = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);
        setSuccess(null);

        // Client-side validation
        if (!email || !password) {
            setError('Email and password are required');
            setLoading(false);
            return;
        }
        if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
            setError('Please enter a valid email address');
            setLoading(false);
            return;
        }

        try {
            const response = await axios.post(
                `${import.meta.env.VITE_API_URL}/api/auth/login`,
                { email, password },
                { headers: { 'Content-Type': 'application/json' } }
            );
            const storage = rememberMe ? localStorage : sessionStorage;
            storage.setItem('jwt_token', response.data.token);
            setAuth(true);
            navigate('/dashboard');
        } catch (err) {
            const message = err.response?.data?.message;
            setError(message === 'Invalid credentials' ? 'Invalid email or password' : message || 'Login failed. Please try again.');
        } finally {
            setLoading(false);
        }
    };

    const handleGoogleLogin = async (credentialResponse) => {
        setLoading(true);
        setError(null);
        setSuccess(null);

        try {
            const response = await axios.post(
                `${import.meta.env.VITE_API_URL}/api/auth/social_login`,
                { id_token: credentialResponse.credential, provider: 'google' },
                { headers: { 'Content-Type': 'application/json' } }
            );
            const storage = rememberMe ? localStorage : sessionStorage;
            storage.setItem('jwt_token', response.data.token);
            setAuth(true);
            navigate('/dashboard');
        } catch (err) {
            setError(err.response?.data?.message || 'Google login failed. Please try again.');
        } finally {
            setLoading(false);
        }
    };

    const handleForgotPassword = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);
        setSuccess(null);

        if (!email) {
            setError('Please enter your email address');
            setLoading(false);
            return;
        }
        if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
            setError('Please enter a valid email address');
            setLoading(false);
            return;
        }

        try {
            await axios.post(
                `${import.meta.env.VITE_API_URL}/api/auth/forgot_password`,
                { email },
                { headers: { 'Content-Type': 'application/json' } }
            );
            setSuccess('Password reset link sent! Check your email.');
        } catch (err) {
            setError(err.response?.data?.message || 'Failed to send reset link. Please try again.');
        } finally {
            setLoading(false);
        }
    };

    const handleResetPassword = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);
        setSuccess(null);

        if (!newPassword || !confirmNewPassword) {
            setError('Please fill in all password fields');
            setLoading(false);
            return;
        }
        if (newPassword.length < 8) {
            setError('Password must be at least 8 characters long');
            setLoading(false);
            return;
        }
        if (newPassword !== confirmNewPassword) {
            setError('Passwords do not match');
            setLoading(false);
            return;
        }

        try {
            await axios.post(
                `${import.meta.env.VITE_API_URL}/api/auth/reset_password`,
                { email, token: resetToken, new_password: newPassword },
                { headers: { 'Content-Type': 'application/json' } }
            );
            setSuccess('Password reset successful! You can now log in.');
            setShowForgotPassword(false);
            setEmail('');
            setNewPassword('');
            setConfirmNewPassword('');
            setResetToken('');
            navigate('/login');
        } catch (err) {
            setError(err.response?.data?.message || 'Password reset failed. Please try again.');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="max-w-md mx-auto mt-8 sm:mt-16 animate-fade-in">
            {/* Glassmorphism Card */}
            <div className="card-glass p-8 sm:p-10">
                {/* Logo and Header */}
                <div className="text-center mb-8">
                    <div className="relative inline-block mb-6">
                        <div className="absolute inset-0 bg-gradient-to-r from-purple-600 to-blue-600 rounded-2xl blur-lg opacity-50 animate-pulse-slow"></div>
                        <div className="relative w-16 h-16 bg-gradient-to-br from-purple-600 via-blue-600 to-indigo-600 rounded-2xl flex items-center justify-center shadow-2xl">
                            <span className="text-white font-black text-2xl">P</span>
                        </div>
                    </div>

                    <h2 className="text-3xl sm:text-4xl font-black text-gray-900 mb-3">
                        {showForgotPassword && resetToken ? (
                            'Reset Password'
                        ) : showForgotPassword ? (
                            'Forgot Password?'
                        ) : (
                            <>Welcome <span className="gradient-text">Back</span></>
                        )}
                    </h2>

                    <p className="text-gray-600">
                        {showForgotPassword && resetToken
                            ? 'Enter your new password below'
                            : showForgotPassword
                                ? 'We\'ll send you a reset link'
                                : 'Sign in to continue to Payego'}
                    </p>
                </div>

                {!showForgotPassword ? (
                    <>
                        {/* Login Form */}
                        <form onSubmit={handleLogin} className="space-y-5">
                            {/* Email Field */}
                            <div>
                                <label htmlFor="email" className="input-label">
                                    Email Address
                                </label>
                                <div className="input-group">
                                    <div className="input-icon">
                                        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 12a4 4 0 10-8 0 4 4 0 008 0zm0 0v1.5a2.5 2.5 0 005 0V12a9 9 0 10-9 9m4.5-1.206a8.959 8.959 0 01-4.5 1.207" />
                                        </svg>
                                    </div>
                                    <input
                                        id="email"
                                        type="email"
                                        value={email}
                                        onChange={(e) => setEmail(e.target.value)}
                                        className="input-with-icon"
                                        placeholder="you@example.com"
                                        required
                                    />
                                </div>
                            </div>

                            {/* Password Field */}
                            <div>
                                <label htmlFor="password" className="input-label">
                                    Password
                                </label>
                                <div className="input-group">
                                    <div className="input-icon">
                                        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                                        </svg>
                                    </div>
                                    <input
                                        id="password"
                                        type={showPassword ? 'text' : 'password'}
                                        value={password}
                                        onChange={(e) => setPassword(e.target.value)}
                                        className="input-with-icon pr-12"
                                        placeholder="••••••••"
                                        required
                                    />
                                    <button
                                        type="button"
                                        onClick={togglePasswordVisibility}
                                        className="absolute right-4 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-purple-600 transition-colors"
                                        aria-label={showPassword ? 'Hide password' : 'Show password'}
                                    >
                                        {showPassword ? <EyeSlashIcon className="h-5 w-5" /> : <EyeIcon className="h-5 w-5" />}
                                    </button>
                                </div>
                            </div>

                            {/* Remember Me & Forgot Password */}
                            <div className="flex items-center justify-between">
                                <label className="flex items-center cursor-pointer group">
                                    <input
                                        id="remember-me"
                                        type="checkbox"
                                        checked={rememberMe}
                                        onChange={(e) => setRememberMe(e.target.checked)}
                                        className="w-4 h-4 text-purple-600 border-gray-300 rounded focus:ring-purple-500 focus:ring-2"
                                    />
                                    <span className="ml-2 text-sm text-gray-700 group-hover:text-gray-900">Remember me</span>
                                </label>

                                <button
                                    type="button"
                                    onClick={() => setShowForgotPassword(true)}
                                    className="text-sm font-semibold gradient-text hover:opacity-80 transition-opacity"
                                >
                                    Forgot password?
                                </button>
                            </div>

                            {/* Submit Button */}
                            <button
                                type="submit"
                                disabled={loading}
                                className="w-full btn-primary-glow btn-lg"
                            >
                                {loading ? (
                                    <span className="flex items-center justify-center space-x-2">
                                        <div className="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                                        <span>Signing in...</span>
                                    </span>
                                ) : (
                                    <span className="flex items-center justify-center space-x-2">
                                        <span>Sign In</span>
                                        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
                                        </svg>
                                    </span>
                                )}
                            </button>

                            {/* Error/Success Messages */}
                            {error && (
                                <div className="alert-error animate-slide-down">
                                    <svg className="w-5 h-5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                                        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                                    </svg>
                                    <span className="text-sm font-medium">{error}</span>
                                </div>
                            )}
                            {success && (
                                <div className="alert-success animate-slide-down">
                                    <svg className="w-5 h-5 flex-shrink-0" fill="currentColor" viewBox="0 0 20 20">
                                        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                                    </svg>
                                    <span className="text-sm font-medium">{success}</span>
                                </div>
                            )}
                        </form>

                        {/* Divider */}
                        <div className="relative my-8">
                            <div className="absolute inset-0 flex items-center">
                                <div className="w-full border-t border-gray-300"></div>
                            </div>
                            <div className="relative flex justify-center text-sm">
                                <span className="px-4 bg-white/80 text-gray-600 font-medium">Or continue with</span>
                            </div>
                        </div>

                        {/* Google Login */}
                        <div className="space-y-3">
                            <GoogleLogin
                                onSuccess={handleGoogleLogin}
                                onError={() => setError('Google login failed. Please try again.')}
                                width="100%"
                                text="continue_with"
                                theme="outline"
                                size="large"
                            />
                        </div>
                    </>
                ) : resetToken ? (
                    // Reset Password Form
                    <form onSubmit={handleResetPassword} className="space-y-5">
                        <div>
                            <label htmlFor="new-password" className="input-label">
                                New Password
                            </label>
                            <div className="input-group">
                                <div className="input-icon">
                                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                                    </svg>
                                </div>
                                <input
                                    id="new-password"
                                    type={showNewPassword ? 'text' : 'password'}
                                    value={newPassword}
                                    onChange={(e) => setNewPassword(e.target.value)}
                                    className="input-with-icon pr-12"
                                    placeholder="••••••••"
                                    required
                                />
                                <button
                                    type="button"
                                    onClick={toggleNewPasswordVisibility}
                                    className="absolute right-4 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-purple-600 transition-colors"
                                >
                                    {showNewPassword ? <EyeSlashIcon className="h-5 w-5" /> : <EyeIcon className="h-5 w-5" />}
                                </button>
                            </div>
                            <p className="mt-1 text-xs text-gray-500">Must be at least 8 characters</p>
                        </div>

                        <div>
                            <label htmlFor="confirm-new-password" className="input-label">
                                Confirm New Password
                            </label>
                            <div className="input-group">
                                <div className="input-icon">
                                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                    </svg>
                                </div>
                                <input
                                    id="confirm-new-password"
                                    type={showConfirmNewPassword ? 'text' : 'password'}
                                    value={confirmNewPassword}
                                    onChange={(e) => setConfirmNewPassword(e.target.value)}
                                    className="input-with-icon pr-12"
                                    placeholder="••••••••"
                                    required
                                />
                                <button
                                    type="button"
                                    onClick={toggleConfirmNewPasswordVisibility}
                                    className="absolute right-4 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-purple-600 transition-colors"
                                >
                                    {showConfirmNewPassword ? <EyeSlashIcon className="h-5 w-5" /> : <EyeIcon className="h-5 w-5" />}
                                </button>
                            </div>
                        </div>

                        <button type="submit" disabled={loading} className="w-full btn-primary-glow btn-lg">
                            {loading ? 'Resetting...' : 'Reset Password'}
                        </button>

                        {error && <div className="alert-error animate-slide-down"><span className="text-sm">{error}</span></div>}
                        {success && <div className="alert-success animate-slide-down"><span className="text-sm">{success}</span></div>}

                        <button
                            type="button"
                            onClick={() => {
                                setShowForgotPassword(false);
                                setResetToken('');
                                setNewPassword('');
                                setConfirmNewPassword('');
                                navigate('/login');
                            }}
                            className="w-full btn-secondary"
                        >
                            Back to Login
                        </button>
                    </form>
                ) : (
                    // Forgot Password Form
                    <form onSubmit={handleForgotPassword} className="space-y-5">
                        <div>
                            <label htmlFor="email" className="input-label">
                                Email Address
                            </label>
                            <div className="input-group">
                                <div className="input-icon">
                                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 12a4 4 0 10-8 0 4 4 0 008 0zm0 0v1.5a2.5 2.5 0 005 0V12a9 9 0 10-9 9m4.5-1.206a8.959 8.959 0 01-4.5 1.207" />
                                    </svg>
                                </div>
                                <input
                                    id="email"
                                    type="email"
                                    value={email}
                                    onChange={(e) => setEmail(e.target.value)}
                                    className="input-with-icon"
                                    placeholder="you@example.com"
                                    required
                                />
                            </div>
                        </div>

                        <button type="submit" disabled={loading} className="w-full btn-primary-glow btn-lg">
                            {loading ? 'Sending...' : 'Send Reset Link'}
                        </button>

                        {error && <div className="alert-error animate-slide-down"><span className="text-sm">{error}</span></div>}
                        {success && <div className="alert-success animate-slide-down"><span className="text-sm">{success}</span></div>}

                        <button
                            type="button"
                            onClick={() => {
                                setShowForgotPassword(false);
                                setEmail('');
                                navigate('/login');
                            }}
                            className="w-full btn-secondary"
                        >
                            Back to Login
                        </button>
                    </form>
                )}

                {/* Sign Up Link */}
                {!showForgotPassword && (
                    <p className="mt-8 text-center text-gray-600">
                        Don't have an account?{' '}
                        <Link to="/register" className="font-bold gradient-text hover:opacity-80 transition-opacity">
                            Create Account
                        </Link>
                    </p>
                )}
            </div>

            {/* Trust Indicators */}
            {!showForgotPassword && (
                <div className="mt-8 flex justify-center items-center space-x-6 text-xs text-gray-500">
                    <div className="flex items-center space-x-1">
                        <svg className="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                            <path fillRule="evenodd" d="M2.166 4.999A11.954 11.954 0 0010 1.944 11.954 11.954 0 0017.834 5c.11.65.166 1.32.166 2.001 0 5.225-3.34 9.67-8 11.317C5.34 16.67 2 12.225 2 7c0-.682.057-1.35.166-2.001zm11.541 3.708a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                        </svg>
                        <span>Secure Login</span>
                    </div>
                    <div className="flex items-center space-x-1">
                        <svg className="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                            <path fillRule="evenodd" d="M5 9V7a5 5 0 0110 0v2a2 2 0 012 2v5a2 2 0 01-2 2H5a2 2 0 01-2-2v-5a2 2 0 012-2zm8-2v2H7V7a3 3 0 016 0z" clipRule="evenodd" />
                        </svg>
                        <span>Encrypted</span>
                    </div>
                </div>
            )}
        </div>
    );
}

export default LoginForm;
