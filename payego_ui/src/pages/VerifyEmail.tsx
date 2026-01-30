import React, { useEffect, useState, useRef } from 'react';
import { useSearchParams, useNavigate } from 'react-router-dom';
import { authApi } from '../api/auth';
import { getErrorMessage } from '../utils/errorHandler';

const VerifyEmail: React.FC = () => {
    const [searchParams] = useSearchParams();
    const [status, setStatus] = useState<'loading' | 'success' | 'error'>('loading');
    const [error, setError] = useState<string | null>(null);
    const navigate = useNavigate();
    const token = searchParams.get('token');
    const hasStartedVerification = useRef(false);

    useEffect(() => {
        const verify = async () => {
            if (!token || hasStartedVerification.current) {
                return;
            }

            hasStartedVerification.current = true;

            try {
                await authApi.verifyEmail(token);
                setStatus('success');
                setTimeout(() => navigate('/dashboard'), 3000);
            } catch (err: any) {
                setStatus('error');
                setError(getErrorMessage(err));
            }
        };

        verify();
    }, [token, navigate]);

    return (
        <div className="min-h-screen flex items-center justify-center bg-gray-50 dark:bg-slate-900 px-4">
            <div className="max-w-md w-full animate-fade-in">
                <div className="card-glass p-8 sm:p-10 text-center">
                    {status === 'loading' && (
                        <>
                            <div className="w-16 h-16 border-4 border-purple-500/30 border-t-purple-600 rounded-full animate-spin mx-auto mb-6"></div>
                            <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-2">Verifying Email...</h2>
                            <p className="text-gray-600 dark:text-slate-400">Please wait while we confirm your email address.</p>
                        </>
                    )}

                    {status === 'success' && (
                        <>
                            <div className="w-20 h-20 bg-green-100 dark:bg-green-900/30 rounded-full flex items-center justify-center mx-auto mb-6 shadow-glow-green animate-bounce-short">
                                <svg className="w-10 h-10 text-green-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M5 13l4 4L19 7" />
                                </svg>
                            </div>
                            <h2 className="text-3xl font-black text-gray-900 dark:text-white mb-4">Verification <span className="gradient-text">Successful!</span></h2>
                            <p className="text-gray-600 dark:text-slate-400 mb-6">
                                Your email has been verified. You're being redirected to your dashboard.
                            </p>
                            <button onClick={() => navigate('/dashboard')} className="btn-primary-glow w-full py-3 rounded-xl font-bold">
                                Go Now
                            </button>
                        </>
                    )}

                    {status === 'error' && (
                        <>
                            <div className="w-20 h-20 bg-red-100 dark:bg-red-900/30 rounded-full flex items-center justify-center mx-auto mb-6">
                                <svg className="w-10 h-10 text-red-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M6 18L18 6M6 6l12 12" />
                                </svg>
                            </div>
                            <h2 className="text-2xl font-bold text-gray-900 dark:text-white mb-4">Verification Failed</h2>
                            <div className="alert-error mb-6">
                                <span className="text-sm">{error || 'Unknown error occurred'}</span>
                            </div>
                            <button onClick={() => navigate('/login')} className="btn-outline w-full py-3 rounded-xl font-bold">
                                Back to Login
                            </button>
                        </>
                    )}
                </div>
            </div>
        </div>
    );
};

export default VerifyEmail;
