
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
    const [loading, setLoading] = useState(false);
    const [passwordStrength, setPasswordStrength] = useState({ score: 0, label: '' });
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
            label = 'Weak';
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

    // Toggle password visibility
    const togglePasswordVisibility = () => {
        setShowPassword(!showPassword);
    };

    // Toggle confirm password visibility
    const toggleConfirmPasswordVisibility = () => {
        setShowConfirmPassword(!showConfirmPassword);
    };

    const handleRegister = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);

        // Client-side validation
        if (!email || !password || !confirmPassword) {
            setError('All fields are required');
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
                `${import.meta.env.VITE_API_URL}/api/register`,
                {
                    email,
                    password,
                    username: username || undefined,
                },
                { headers: { 'Content-Type': 'application/json' } }
            );
            localStorage.setItem('jwt_token', response.data.token);
            setShowVerification(true); // Show verification input
        } catch (err) {
            setError(err.response?.data?.message || 'Registration failed');
            setLoading(false);
        }
    };

    const handleVerifyEmail = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);

        try {
            const response = await axios.post(
                `${import.meta.env.VITE_API_URL}/api/verify_email`,
                {
                    email,
                    code: verificationCode,
                },
                { headers: { 'Content-Type': 'application/json' } }
            );
            setAuth(true);
            navigate('/');
        } catch (err) {
            setError(err.response?.data?.message || 'Verification failed');
            setLoading(false);
        }
    };

    const handleResendCode = async () => {
        setLoading(true);
        setError(null);

        try {
            await axios.post(
                `${import.meta.env.VITE_API_URL}/api/send_verification`,
                { email },
                { headers: { 'Content-Type': 'application/json' } }
            );
            setError('Verification code resent');
        } catch (err) {
            setError(err.response?.data?.message || 'Failed to resend code');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="max-w-md mx-auto mt-10 p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
            <div className="text-center mb-8">
                <div className="w-16 h-16 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-2xl flex items-center justify-center mx-auto mb-4">
                    <span className="text-white font-bold text-xl">P</span>
                </div>
                <h2 className="text-3xl font-bold text-gray-800 mb-2">
                    {showVerification ? 'Verify Email' : 'Create Account'}
                </h2>
                <p className="text-gray-600">
                    {showVerification ? 'Enter the verification code sent to your email' : 'Join Payego and start managing your finances'}
                </p>
            </div>
            {!showVerification ? (
                <form onSubmit={handleRegister}>
                    <div className="mb-4">
                        <label className="block text-gray-700 font-medium mb-2">Email Address</label>
                        <input
                            type="email"
                            value={email}
                            onChange={(e) => setEmail(e.target.value)}
                            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                            placeholder="Enter your email"
                            required
                        />
                    </div>
                    <div className="mb-4">
                        <label className="block text-gray-700 font-medium mb-2">Password</label>
                        <div className="relative">
                            <input
                                type={showPassword ? 'text' : 'password'}
                                value={password}
                                onChange={handlePasswordChange}
                                className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                                placeholder="Create a password"
                                required
                            />
                            <button
                                type="button"
                                onClick={togglePasswordVisibility}
                                className="absolute inset-y-0 right-0 flex items-center pr-3 text-gray-600 hover:text-blue-600 transition-colors duration-200"
                            >
                                {showPassword ? <EyeSlashIcon className="h-5 w-5" /> : <EyeIcon className="h-5 w-5" />}
                            </button>
                        </div>
                        {password && (
                            <div className="mt-2">
                                <div className="w-full bg-gray-200 rounded-full h-2">
                                    <div
                                        className={`h-2 rounded-full transition-all duration-200 ${passwordStrength.color}`}
                                        style={{ width: `${(passwordStrength.score / 5) * 100}%` }}
                                    ></div>
                                </div>
                                <p className="text-sm text-gray-600 mt-1">
                                    Password Strength: <span className={passwordStrength.color.replace('bg-', 'text-')}>{passwordStrength.label}</span>
                                </p>
                            </div>
                        )}
                    </div>
                    <div className="mb-4">
                        <label className="block text-gray-700 font-medium mb-2">Confirm Password</label>
                        <div className="relative">
                            <input
                                type={showConfirmPassword ? 'text' : 'password'}
                                value={confirmPassword}
                                onChange={(e) => setConfirmPassword(e.target.value)}
                                className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                                placeholder="Confirm your password"
                                required
                            />
                            <button
                                type="button"
                                onClick={toggleConfirmPasswordVisibility}
                                className="absolute inset-y-0 right-0 flex items-center pr-3 text-gray-600 hover:text-blue-600 transition-colors duration-200"
                            >
                                {showConfirmPassword ? <EyeSlashIcon className="h-5 w-5" /> : <EyeIcon className="h-5 w-5" />}
                            </button>
                        </div>
                    </div>
                    <div className="mb-4">
                        <label className="block text-gray-700 font-medium mb-2">Username (optional)</label>
                        <input
                            type="text"
                            value={username}
                            onChange={(e) => setUsername(e.target.value)}
                            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                            placeholder="Enter your username"
                        />
                    </div>
                    <button
                        type="submit"
                        disabled={loading}
                        className="w-full bg-gradient-to-r from-blue-600 to-indigo-600 text-white p-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 disabled:from-gray-400 disabled:to-gray-400 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                    >
                        {loading ? 'Registering...' : 'Register'}
                    </button>
                    {error && (
                        <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                            <p className="text-red-600 text-center text-sm">{error}</p>
                        </div>
                    )}
                </form>
            ) : (
                <form onSubmit={handleVerifyEmail}>
                    <div className="mb-4">
                        <label className="block text-gray-700 font-medium mb-2">Verification Code</label>
                        <input
                            type="text"
                            value={verificationCode}
                            onChange={(e) => setVerificationCode(e.target.value)}
                            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                            placeholder="Enter 6-digit code"
                            required
                        />
                    </div>
                    <button
                        type="submit"
                        disabled={loading}
                        className="w-full bg-gradient-to-r from-blue-600 to-indigo-600 text-white p-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 disabled:from-gray-400 disabled:to-gray-400 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                    >
                        {loading ? 'Verifying...' : 'Verify Email'}
                    </button>
                    <button
                        type="button"
                        onClick={handleResendCode}
                        disabled={loading}
                        className="w-full mt-4 bg-gray-200 text-gray-700 p-3 rounded-lg hover:bg-gray-300 disabled:bg-gray-400 transition-all duration-200 font-medium"
                    >
                        {loading ? 'Resending...' : 'Resend Code'}
                    </button>
                    {error && (
                        <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                            <p className="text-red-600 text-center text-sm">{error}</p>
                        </div>
                    )}
                </form>
            )}
            <p className="mt-6 text-center text-gray-600">
                Already have an account?{' '}
                <Link to="/login" className="text-blue-600 hover:text-blue-700 font-medium transition-colors duration-200">
                    Sign In
                </Link>
            </p>
        </div>
    );
}

export default RegisterForm;
