import React, { useEffect, useState } from 'react';
import { authApi } from '../api/auth';
import { getErrorMessage } from '../utils/errorHandler';
import { useAuth } from '../contexts/AuthContext';

interface AuditLog {
    id: string;
    event_type: string;
    metadata: any;
    created_at: string;
    ip_address?: string;
}

const Security: React.FC = () => {
    const { user } = useAuth();
    const [logs, setLogs] = useState<AuditLog[]>([]);
    const [loading, setLoading] = useState(true);
    const [error, setError] = useState<string | null>(null);
    const [resending, setResending] = useState(false);
    const [resendSuccess, setResendSuccess] = useState(false);

    useEffect(() => {
        fetchLogs();
    }, []);

    const fetchLogs = async () => {
        try {
            setLoading(true);
            const response = await authApi.getAuditLogs(1, 20);
            setLogs(response.data.data || []);
        } catch (err: any) {
            setError(getErrorMessage(err));
        } finally {
            setLoading(false);
        }
    };

    const handleResend = async () => {
        try {
            setResending(true);
            await authApi.resendVerification();
            setResendSuccess(true);
            setTimeout(() => setResendSuccess(false), 5000);
        } catch (err: any) {
            setError(getErrorMessage(err));
        } finally {
            setResending(false);
        }
    };

    const isVerified = !!user?.email_verified_at;

    return (
        <div className="max-w-4xl mx-auto space-y-8 animate-fade-in">
            {/* Account Status Header */}
            <div className="card-glass p-8 relative overflow-hidden">
                <div className="absolute top-0 right-0 w-32 h-32 bg-gradient-to-br from-purple-600/10 to-blue-600/10 rounded-full -mr-16 -mt-16 blur-2xl"></div>
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-6">
                    <div>
                        <h2 className="text-3xl font-black text-gray-900 dark:text-white mb-2">Account <span className="gradient-text">Security</span></h2>
                        <p className="text-gray-600 dark:text-slate-400">Manage your account protection and view recent activity.</p>
                    </div>

                    <div className="flex items-center space-x-4">
                        {isVerified ? (
                            <div className="flex items-center space-x-2 bg-green-500/10 text-green-600 dark:text-green-400 px-4 py-2 rounded-full border border-green-500/20 shadow-glow-green">
                                <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                                <span className="font-bold">Email Verified</span>
                            </div>
                        ) : (
                            <div className="flex flex-col items-end gap-2">
                                <div className="flex items-center space-x-2 bg-yellow-500/10 text-yellow-600 px-4 py-2 rounded-full border border-yellow-500/20">
                                    <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                                    </svg>
                                    <span className="font-bold text-sm">Action Needed: Verify Email</span>
                                </div>
                                <button
                                    onClick={handleResend}
                                    disabled={resending}
                                    className="text-xs font-bold text-blue-600 hover:text-blue-700 transition-colors uppercase tracking-wider"
                                >
                                    {resending ? 'Sending...' : 'Resend link'}
                                </button>
                                {resendSuccess && <span className="text-[10px] text-green-500 font-bold">Resent successfully!</span>}
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {error && (
                <div className="alert-error">
                    <p className="text-sm">{error}</p>
                </div>
            )}

            {/* Activity Feed */}
            <div className="card-glass overflow-hidden shadow-2xl">
                <div className="px-8 py-6 border-b border-gray-200/50 dark:border-slate-800/50 flex justify-between items-center">
                    <h3 className="text-xl font-bold text-gray-900 dark:text-white">Recent Activity</h3>
                    <button onClick={fetchLogs} className="text-gray-400 hover:text-blue-500 transition-colors">
                        <svg className={`w-5 h-5 ${loading ? 'animate-spin' : ''}`} fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
                        </svg>
                    </button>
                </div>

                <div className="divide-y divide-gray-200/50 dark:divide-slate-800/50">
                    {loading && logs.length === 0 ? (
                        <div className="p-12 text-center">
                            <div className="w-12 h-12 border-4 border-blue-500/20 border-t-blue-600 rounded-full animate-spin mx-auto mb-4"></div>
                            <p className="text-gray-500 font-medium">Crunching your logs...</p>
                        </div>
                    ) : logs.length === 0 ? (
                        <div className="p-12 text-center text-gray-500 font-medium">
                            No security events recorded yet.
                        </div>
                    ) : (
                        logs.map((log) => (
                            <div key={log.id} className="px-8 py-6 hover:bg-gray-50/50 dark:hover:bg-slate-800/30 transition-colors group">
                                <div className="flex items-start justify-between">
                                    <div className="space-y-1">
                                        <div className="flex items-center gap-3">
                                            <span className="text-lg">{getEventIcon(log.event_type)}</span>
                                            <span className="font-bold text-gray-900 dark:text-white group-hover:text-blue-600 transition-colors">
                                                {formatEventName(log.event_type)}
                                            </span>
                                        </div>
                                        <p className="text-sm text-gray-500 dark:text-slate-400 font-medium">
                                            {new Date(log.created_at).toLocaleString()}
                                            {log.ip_address && ` • IP: ${log.ip_address}`}
                                        </p>
                                    </div>
                                    <div className="text-right">
                                        {/* Optional details placeholder */}
                                    </div>
                                </div>
                            </div>
                        ))
                    )}
                </div>
            </div>

            {/* Enhanced Protection Section */}
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <div className="card-glass p-6 border-l-4 border-blue-600">
                    <h4 className="font-bold text-lg mb-2 text-gray-900 dark:text-white">Transaction Protection</h4>
                    <p className="text-gray-600 dark:text-slate-400 text-sm mb-4">Unverified accounts are restricted from making transfers and withdrawals to ensure fund safety.</p>
                </div>
                <div className="card-glass p-6 border-l-4 border-purple-600">
                    <h4 className="font-bold text-lg mb-2 text-gray-900 dark:text-white">Automated Audit</h4>
                    <p className="text-gray-600 dark:text-slate-400 text-sm mb-4">Every security-critical event is cryptographically logged for your protection and review.</p>
                </div>
            </div>
        </div>
    );
};

const getEventIcon = (type: string) => {
    if (type.includes('login.success')) return '🔐';
    if (type.includes('login.failure')) return '❌';
    if (type.includes('register')) return '👋';
    if (type.includes('transfer')) return '💸';
    if (type.includes('withdraw')) return '🏦';
    if (type.includes('conversion')) return '🔄';
    if (type.includes('wallet')) return '💳';
    if (type.includes('bank')) return '🏛️';
    return '📝';
};

const formatEventName = (type: string) => {
    return type
        .split('.')
        .map(s => s.charAt(0).toUpperCase() + s.slice(1))
        .join(' ');
};

export default Security;
