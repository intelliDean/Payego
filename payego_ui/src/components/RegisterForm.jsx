import React, { useState } from 'react';
import axios from 'axios';
import { Link, useNavigate } from 'react-router-dom';
import { EyeIcon, EyeSlashIcon } from '@heroicons/react/24/outline';

function RegisterForm({ setAuth }) {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [confirmPassword, setConfirmPassword] = useState('');
    const [username, setUsername] = useState('');
    const [error, setError] = useState(null);
    const [success, setSuccess] = useState(null);
    const [loading, setLoading] = useState(false);
    const [passwordStrength, setPasswordStrength] = useState({ score: 0, label: '', color: '' });
    const [showPassword, setShowPassword] = useState(false);
    const [showConfirmPassword, setShowConfirmPassword] = useState(false);
    const [verificationCode, setVerificationCode] = useState('');
    const [showVerification, setShowVerification] = useState(false);
    const navigate = useNavigate();

    // Password strength evaluation
    const evaluatePasswordStrength = (password) => {
        let score = 0;
        const checks = [
            { regex: /.{8,}/, points: 1 }, // At least 8 characters
            { regex: /[A-Z]/, points: 1 }, // Uppercase letter
            { regex: /[a-z]/, points: 1 }, // Lowercase letter
            { regex: /[0-9]/, points: 1 }, // Number
            { regex: /[^A-Za-z0-9]/, points: 1 }, // Special character
        ];

        checks.forEach(check => {
            if (check.regex.test(password)) {
                score += check.points;
            }
        });

        let label = '';
        let color = '';
        if (score === 0) {
            label = 'Too Weak';
            color = 'bg-red-500';
        } else if (score <= 2) {
            label = 'Weak';
            color = 'bg-red-500';
        } else if (score <= 3) {
            label = 'Fair';
            color = 'bg-yellow-500';
        } else if (score <= 4) {
            label = 'Good';
            color = 'bg-blue-500';
        } else {
            label = 'Strong';
            color = 'bg-green-500';
        }

        return { score, label, color };
    };

    // Update password strength
    const handlePasswordChange = (e) => {
        const newPassword = e.target.value;
        setPassword(newPassword);
        setPasswordStrength(evaluatePasswordStrength(newPassword));
    };

    const handleRegister = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);
        setSuccess(null);

        // Client-side validation
        if (!email || !password || !confirmPassword) {
            setError('All fields are required');
            setLoading(false);
            return;
        }
        if (!/^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(email)) {
            setError('Please enter a valid email address');
            setLoading(false);
            return;
        }
        if (password !== confirmPassword) {
            setError('Passwords do not match');
            setLoading(false);
            return;
        }
        if (username && (username.length < 3 || username.length > 50)) {
            setError('Username must be 3-50 characters if provided');
            setLoading(false);
            return;
        }
        if (password.length < 8) {
            setError('Password must be at least 8 characters');
            setLoading(false);
            return;
        }

        try {
            const response = await axios.post(
                `${import.meta.env.VITE_API_URL}/api/auth/register`,
                {
                    email,
                    password,
                    username: username || undefined,
                },
                { headers: { 'Content-Type': 'application/json' } }
            );
            localStorage.setItem('jwt_token', response.data.token);
            setShowVerification(true);
        } catch (err) {
            setError(err.response?.data?.message || 'Registration failed. Please try again.');
            setLoading(false);
        }
    };

    const handleVerifyEmail = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);
        setSuccess(null);

        try {
            await axios.post(
                `${import.meta.env.VITE_API_URL}/api/verify_email`,
                {
                    email,
                    code: verificationCode,
                },
                { headers: { 'Content-Type': 'application/json' } }
            );
            setAuth(true);
            navigate('/dashboard');
        } catch (err) {
            setError(err.response?.data?.message || 'Verification failed. Please try again.');
            setLoading(false);
        }
    };

    const handleResendCode = async () => {
        setLoading(true);
        setError(null);
        setSuccess(null);

        try {
            await axios.post(
                `${import.meta.env.VITE_API_URL}/api/send_verification`,
                { email },
                { headers: { 'Content-Type': 'application/json' } }
            );
            setSuccess('Verification code resent! Check your email.');
        } catch (err) {
            setError(err.response?.data?.message || 'Failed to resend code');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="max-w-md mx-auto mt-8 sm:mt-16 animate-fade-in">
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
                        {showVerification ? (
                            <>Verify <span className="gradient-text">Email</span></>
                        ) : (
                            <>Create <span className="gradient-text">Account</span></>
                        )}
                    </h2>

                    <p className="text-gray-600">
                        {showVerification
                            ? 'Enter the code sent to your email'
                            : 'Join Payego and start managing your finances'}
                    </p>
                </div>

                {!showVerification ? (
                    <form onSubmit={handleRegister} className="space-y-5">
                        {/* Email Field */}
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
                                    value={email}
                                    onChange={(e) => setEmail(e.target.value)}
                                    className="input-with-icon"
                                    placeholder="you@example.com"
                                    required
                                />
                            </div>
                        </div>

                        {/* Username Field */}
                        <div>
                            <label htmlFor="username" className="input-label">
                                Username <span className="text-gray-400 text-xs">(optional)</span>
                            </label>
                            <div className="input-group">
                                <div className="input-icon">
                                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M16 7a4 4 0 11-8 0 4 4 0 018 0zM12 14a7 7 0 00-7 7h14a7 7 0 00-7-7z" />
                                    </svg>
                                </div>
                                <input
                                    id="username"
                                    type="text"
                                    value={username}
                                    onChange={(e) => setUsername(e.target.value)}
                                    className="input-with-icon"
                                    placeholder="johndoe"
                                />
                            </div>
                        </div>

                        {/* Password Field */}
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
                                    value={password}
                                    onChange={handlePasswordChange}
                                    className="input-with-icon pr-12"
                                    placeholder="••••••••"
                                    required
                                />
                                <button
                                    type="button"
                                    onClick={() => setShowPassword(!showPassword)}
                                    className="absolute right-4 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-purple-600 transition-colors"
                                >
                                    {showPassword ? <EyeSlashIcon className="h-5 w-5" /> : <EyeIcon className="h-5 w-5" />}
                                </button>
                            </div>
                            {password && (
                                <div className="mt-2">
                                    <div className="w-full bg-gray-200 rounded-full h-2 overflow-hidden">
                                        <div
                                            className={`h-2 rounded-full transition-all duration-300 ${passwordStrength.color}`}
                                            style={{ width: `${(passwordStrength.score / 5) * 100}%` }}
                                        ></div>
                                    </div>
                                    <p className="text-xs text-gray-600 mt-1">
                                        Strength: <span className={`font-semibold ${passwordStrength.color.replace('bg-', 'text-')}`}>{passwordStrength.label}</span>
                                    </p>
                                </div>
                            )}
                        </div>

                        {/* Confirm Password Field */}
                        <div>
                            <label htmlFor="confirmPassword" className="input-label">Confirm Password</label>
                            <div className="input-group">
                                <div className="input-icon">
                                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                    </svg>
                                </div>
                                <input
                                    id="confirmPassword"
                                    type={showConfirmPassword ? 'text' : 'password'}
                                    value={confirmPassword}
                                    onChange={(e) => setConfirmPassword(e.target.value)}
                                    className="input-with-icon pr-12"
                                    placeholder="••••••••"
                                    required
                                />
                                <button
                                    type="button"
                                    onClick={() => setShowConfirmPassword(!showConfirmPassword)}
                                    className="absolute right-4 top-1/2 transform -translate-y-1/2 text-gray-400 hover:text-purple-600 transition-colors"
                                >
                                    {showConfirmPassword ? <EyeSlashIcon className="h-5 w-5" /> : <EyeIcon className="h-5 w-5" />}
                                </button>
                            </div>
                        </div>

                        {/* Submit Button */}
                        <button type="submit" disabled={loading} className="w-full btn-primary-glow btn-lg">
                            {loading ? (
                                <span className="flex items-center justify-center space-x-2">
                                    <div className="w-5 h-5 border-2 border-white border-t-transparent rounded-full animate-spin"></div>
                                    <span>Creating Account...</span>
                                </span>
                            ) : (
                                'Create Account'
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
                ) : (
                    <form onSubmit={handleVerifyEmail} className="space-y-5">
                        <div>
                            <label htmlFor="verificationCode" className="input-label">Verification Code</label>
                            <div className="input-group">
                                <div className="input-icon">
                                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                    </svg>
                                </div>
                                <input
                                    id="verificationCode"
                                    type="text"
                                    value={verificationCode}
                                    onChange={(e) => setVerificationCode(e.target.value)}
                                    className="input-with-icon text-center tracking-widest text-lg"
                                    placeholder="000000"
                                    maxLength="6"
                                    required
                                />
                            </div>
                        </div>

                        <button type="submit" disabled={loading} className="w-full btn-primary-glow btn-lg">
                            {loading ? 'Verifying...' : 'Verify Email'}
                        </button>

                        <button
                            type="button"
                            onClick={handleResendCode}
                            disabled={loading}
                            className="w-full btn-secondary"
                        >
                            Resend Code
                        </button>

                        {error && <div className="alert-error animate-slide-down"><span className="text-sm">{error}</span></div>}
                        {success && <div className="alert-success animate-slide-down"><span className="text-sm">{success}</span></div>}
                    </form>
                )}

                {/* Sign In Link */}
                <p className="mt-8 text-center text-gray-600">
                    Already have an account?{' '}
                    <Link to="/login" className="font-bold gradient-text hover:opacity-80 transition-opacity">
                        Sign In
                    </Link>
                </p>
            </div>

            {/* Trust Indicators */}
            {!showVerification && (
                <div className="mt-8 flex justify-center items-center space-x-6 text-xs text-gray-500">
                    <div className="flex items-center space-x-1">
                        <svg className="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                            <path fillRule="evenodd" d="M2.166 4.999A11.954 11.954 0 0010 1.944 11.954 11.954 0 0017.834 5c.11.65.166 1.32.166 2.001 0 5.225-3.34 9.67-8 11.317C5.34 16.67 2 12.225 2 7c0-.682.057-1.35.166-2.001zm11.541 3.708a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                        </svg>
                        <span>Free Forever</span>
                    </div>
                    <div className="flex items-center space-x-1">
                        <svg className="w-4 h-4 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                            <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                        </svg>
                        <span>No Credit Card</span>
                    </div>
                </div>
            )}
        </div>
    );
}

export default RegisterForm;
