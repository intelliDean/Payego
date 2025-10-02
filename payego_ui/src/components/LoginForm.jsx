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

        // Client-side validation
        if (!email || !password) {
            setError('Email/password missing. Don’t ghost us!');
            setLoading(false);
            return;
        }
        if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
            setError('Email’s wonky. Try a real one!');
            setLoading(false);
            return;
        }

        try {
            const response = await axios.post(
                `${import.meta.env.VITE_API_URL}/api/login`,
                { email, password },
                { headers: { 'Content-Type': 'application/json' } }
            );
            const storage = rememberMe ? localStorage : sessionStorage;
            storage.setItem('jwt_token', response.data.token);
            setAuth(true);
            navigate('/dashboard');
        } catch (err) {
            const message = err.response?.data?.message;
            setError(message === 'Invalid credentials' ? 'Oops, wrong keys to the Payego castle!' : message || 'Login crashed. Try again!');
        } finally {
            setLoading(false);
        }
    };

    const handleGoogleLogin = async (credentialResponse) => {
        setLoading(true);
        setError(null);

        try {
            const response = await axios.post(
                `${import.meta.env.VITE_API_URL}/api/social_login`,
                { id_token: credentialResponse.credential, provider: 'google' },
                { headers: { 'Content-Type': 'application/json' } }
            );
            const storage = rememberMe ? localStorage : sessionStorage;
            storage.setItem('jwt_token', response.data.token);
            setAuth(true);
            navigate('/dashboard');
        } catch (err) {
            setError(err.response?.data?.message || 'Google login flopped. Try again!');
        } finally {
            setLoading(false);
        }
    };

    const handleForgotPassword = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);

        if (!email) {
            setError('Email’s AWOL. Where is it?');
            setLoading(false);
            return;
        }
        if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
            setError('Email’s wonky. Try a real one!');
            setLoading(false);
            return;
        }

        try {
            await axios.post(
                `${import.meta.env.VITE_API_URL}/api/forgot_password`,
                { email },
                { headers: { 'Content-Type': 'application/json' } }
            );
            setError('Reset link sent. Check your inbox!');
        } catch (err) {
            setError(err.response?.data?.message || 'Reset email didn’t fly. Try again!');
        } finally {
            setLoading(false);
        }
    };

    const handleResetPassword = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);

        if (!newPassword || !confirmNewPassword) {
            setError('Passwords missing. Don’t leave us hanging!');
            setLoading(false);
            return;
        }
        if (newPassword.length < 8) {
            setError('Password’s too puny. 8+ characters, please!');
            setLoading(false);
            return;
        }
        if (newPassword !== confirmNewPassword) {
            setError('Passwords don’t vibe. Make ‘em match!');
            setLoading(false);
            return;
        }

        try {
            await axios.post(
                `${import.meta.env.VITE_API_URL}/api/reset_password`,
                { email, token: resetToken, new_password: newPassword },
                { headers: { 'Content-Type': 'application/json' } }
            );
            setError('Password reset! Log in with your new key.');
            setShowForgotPassword(false);
            setEmail('');
            setNewPassword('');
            setConfirmNewPassword('');
            setResetToken('');
            navigate('/login');
        } catch (err) {
            setError(err.response?.data?.message || 'Reset crashed. Try again!');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="max-w-md mx-auto mt-4 sm:mt-10 p-6 sm:p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
            <div className="text-center mb-8">
                <div className="w-12 sm:w-16 h-12 sm:h-16 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-2xl flex items-center justify-center mx-auto mb-4">
                    <span className="text-white font-bold text-lg sm:text-xl">P</span>
                </div>
                <h2 className="text-2xl sm:text-3xl font-bold text-gray-800 mb-2">
                    {showForgotPassword && resetToken ? 'Reset Password' : showForgotPassword ? 'Forgot Password' : 'Welcome Back'}
                </h2>
                <p className="text-sm sm:text-base text-gray-600">
                    {showForgotPassword && resetToken ? 'Enter your new password' : showForgotPassword ? 'Enter your email to receive a reset link' : 'Sign in to your Payego account'}
                </p>
            </div>
            {!showForgotPassword ? (
                <>
                    <form onSubmit={handleLogin}>
                        <div className="mb-4">
                            <label htmlFor="email" className="block text-gray-700 font-medium mb-2">
                                Email Address
                            </label>
                            <input
                                id="email"
                                type="email"
                                value={email}
                                onChange={(e) => setEmail(e.target.value)}
                                className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                                placeholder="Enter your email"
                                required
                                aria-describedby={error && email ? 'email-error' : undefined}
                            />
                        </div>
                        <div className="mb-4">
                            <label htmlFor="password" className="block text-gray-700 font-medium mb-2">
                                Password
                            </label>
                            <div className="relative">
                                <input
                                    id="password"
                                    type={showPassword ? 'text' : 'password'}
                                    value={password}
                                    onChange={(e) => setPassword(e.target.value)}
                                    className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                                    placeholder="Enter your password"
                                    required
                                    aria-describedby={error && password ? 'password-error' : undefined}
                                />
                                <button
                                    type="button"
                                    onClick={togglePasswordVisibility}
                                    className="absolute inset-y-0 right-0 flex items-center pr-3 text-gray-600 hover:text-blue-600 transition-colors duration-200"
                                    aria-label={showPassword ? 'Hide password' : 'Show password'}
                                >
                                    {showPassword ? <EyeSlashIcon className="h-5 w-5" /> : <EyeIcon className="h-5 w-5" />}
                                </button>
                            </div>
                        </div>
                        <div className="mb-4 flex items-center">
                            <input
                                id="remember-me"
                                type="checkbox"
                                checked={rememberMe}
                                onChange={(e) => setRememberMe(e.target.checked)}
                                className="h-4 w-4 text-blue-600 focus:ring-blue-500 border-gray-300 rounded"
                                aria-label="Remember me"
                            />
                            <label htmlFor="remember-me" className="ml-2 text-gray-700 text-sm">
                                Remember me
                            </label>
                        </div>
                        <button
                            type="submit"
                            disabled={loading}
                            className="w-full bg-gradient-to-r from-blue-600 to-indigo-600 text-white p-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 disabled:from-gray-400 disabled:to-gray-400 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                        >
                            {loading ? 'Logging in...' : 'Login'}
                        </button>
                        {error && (
                            <div id="error-message" className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                                <p className="text-red-600 text-center text-sm">{error}</p>
                            </div>
                        )}
                        <p className="mt-4 text-center text-gray-600">
                            <button
                                type="button"
                                onClick={() => setShowForgotPassword(true)}
                                className="text-blue-600 hover:text-blue-700 font-medium transition-colors duration-200"
                            >
                                Forgot Password?
                            </button>
                        </p>
                    </form>
                    <div className="mt-6">
                        <p className="text-center text-gray-600 mb-4">Or sign in with</p>
                        <GoogleLogin
                            onSuccess={handleGoogleLogin}
                            onError={() => setError('Google login flopped. Try again!')}
                            width="100%"
                            text="Sign in with Google"
                            theme="outline"
                            size="large"
                        />
                    </div>
                </>
            ) : resetToken ? (
                <form onSubmit={handleResetPassword}>
                    <div className="mb-4">
                        <label htmlFor="new-password" className="block text-gray-700 font-medium mb-2">
                            New Password
                        </label>
                        <div className="relative">
                            <input
                                id="new-password"
                                type={showNewPassword ? 'text' : 'password'}
                                value={newPassword}
                                onChange={(e) => setNewPassword(e.target.value)}
                                className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                                placeholder="Enter new password"
                                required
                                aria-describedby={error && newPassword ? 'new-password-error' : undefined}
                            />
                            <button
                                type="button"
                                onClick={toggleNewPasswordVisibility}
                                className="absolute inset-y-0 right-0 flex items-center pr-3 text-gray-600 hover:text-blue-600 transition-colors duration-200"
                                aria-label={showNewPassword ? 'Hide new password' : 'Show new password'}
                            >
                                {showNewPassword ? <EyeSlashIcon className="h-5 w-5" /> : <EyeIcon className="h-5 w-5" />}
                            </button>
                        </div>
                    </div>
                    <div className="mb-4">
                        <label htmlFor="confirm-new-password" className="block text-gray-700 font-medium mb-2">
                            Confirm New Password
                        </label>
                        <div className="relative">
                            <input
                                id="confirm-new-password"
                                type={showConfirmNewPassword ? 'text' : 'password'}
                                value={confirmNewPassword}
                                onChange={(e) => setConfirmNewPassword(e.target.value)}
                                className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                                placeholder="Confirm new password"
                                required
                                aria-describedby={error && confirmNewPassword ? 'confirm-new-password-error' : undefined}
                            />
                            <button
                                type="button"
                                onClick={toggleConfirmNewPasswordVisibility}
                                className="absolute inset-y-0 right-0 flex items-center pr-3 text-gray-600 hover:text-blue-600 transition-colors duration-200"
                                aria-label={showConfirmNewPassword ? 'Hide confirm password' : 'Show confirm password'}
                            >
                                {showConfirmNewPassword ? <EyeSlashIcon className="h-5 w-5" /> : <EyeIcon className="h-5 w-5" />}
                            </button>
                        </div>
                    </div>
                    <button
                        type="submit"
                        disabled={loading}
                        className="w-full bg-gradient-to-r from-blue-600 to-indigo-600 text-white p-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 disabled:from-gray-400 disabled:to-gray-400 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                    >
                        {loading ? 'Resetting...' : 'Reset Password'}
                    </button>
                    {error && (
                        <div id="error-message" className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                            <p className="text-red-600 text-center text-sm">{error}</p>
                        </div>
                    )}
                    <p className="mt-4 text-center text-gray-600">
                        <button
                            type="button"
                            onClick={() => {
                                setShowForgotPassword(false);
                                setResetToken('');
                                setNewPassword('');
                                setConfirmNewPassword('');
                                navigate('/login');
                            }}
                            className="text-blue-600 hover:text-blue-700 font-medium transition-colors duration-200"
                        >
                            Back to Login
                        </button>
                    </p>
                </form>
            ) : (
                <form onSubmit={handleForgotPassword}>
                    <div className="mb-4">
                        <label htmlFor="email" className="block text-gray-700 font-medium mb-2">
                            Email Address
                        </label>
                        <input
                            id="email"
                            type="email"
                            value={email}
                            onChange={(e) => setEmail(e.target.value)}
                            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                            placeholder="Enter your email"
                            required
                            aria-describedby={error && email ? 'email-error' : undefined}
                        />
                    </div>
                    <button
                        type="submit"
                        disabled={loading}
                        className="w-full bg-gradient-to-r from-blue-600 to-indigo-600 text-white p-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 disabled:from-gray-400 disabled:to-gray-400 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                    >
                        {loading ? 'Sending...' : 'Send Reset Link'}
                    </button>
                    {error && (
                        <div id="error-message" className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                            <p className="text-red-600 text-center text-sm">{error}</p>
                        </div>
                    )}
                    <p className="mt-4 text-center text-gray-600">
                        <button
                            type="button"
                            onClick={() => {
                                setShowForgotPassword(false);
                                setEmail('');
                                navigate('/login');
                            }}
                            className="text-blue-600 hover:text-blue-700 font-medium transition-colors duration-200"
                        >
                            Back to Login
                        </button>
                    </p>
                </form>
            )}
            {!showForgotPassword && (
                <p className="mt-6 text-center text-gray-600">
                    Don’t have an account?{' '}
                    <Link to="/register" className="text-blue-600 hover:text-blue-700 font-medium transition-colors duration-200">
                        Create Account
                    </Link>
                </p>
            )}
        </div>
    );
}

export default LoginForm;

