import React from 'react';
import { useAuth } from '../contexts/AuthContext';
import ErrorBoundary from './ErrorBoundary';

const Profile: React.FC = () => {
    const { user, logout } = useAuth();

    if (!user) return null;

    const formatDate = (dateStr: string) => {
        return new Date(dateStr).toLocaleDateString("en-US", {
            month: "long",
            day: "numeric",
            year: "numeric",
        });
    };

    const displayName = user.username || user.email.split('@')[0];
    const initial = displayName.charAt(0).toUpperCase();

    return (
        <ErrorBoundary>
            <div className="max-w-4xl mx-auto px-4 py-8 sm:py-12">
                <div className="mb-10 text-center sm:text-left">
                    <h1 className="text-4xl font-black text-gray-900 tracking-tight">Your Profile</h1>
                    <p className="text-gray-500 mt-2 text-lg">Manage your personal settings and account security</p>
                </div>

                <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
                    {/* Left Column: Avatar and Quick Stats */}
                    <div className="lg:col-span-1 space-y-6">
                        <div className="bg-white rounded-3xl shadow-xl shadow-blue-500/5 border border-gray-100 p-8 text-center overflow-hidden relative group">
                            <div className="absolute top-0 left-0 w-full h-2 bg-gradient-to-r from-blue-600 to-indigo-600"></div>
                            <div className="relative inline-block mx-auto mb-6">
                                <div className="w-32 h-32 bg-gradient-to-br from-blue-600 to-indigo-600 rounded-full flex items-center justify-center text-5xl text-white font-black shadow-2xl ring-8 ring-blue-50">
                                    {initial}
                                </div>
                                <div className="absolute bottom-1 right-1 w-8 h-8 bg-green-500 border-4 border-white rounded-full shadow-lg"></div>
                            </div>
                            <h2 className="text-2xl font-bold text-gray-900 break-words">{displayName}</h2>
                            <p className="text-gray-500 font-medium mb-6 break-all">{user.email}</p>

                            <div className="inline-flex items-center px-4 py-1.5 rounded-full bg-green-50 text-green-700 text-sm font-bold border border-green-100">
                                <span className="w-2 h-2 bg-green-500 rounded-full mr-2"></span>
                                Active Account
                            </div>
                        </div>

                        <div className="bg-gradient-to-br from-gray-900 to-blue-900 rounded-3xl p-8 text-white shadow-2xl relative overflow-hidden">
                            <div className="absolute top-0 right-0 w-32 h-32 bg-white/5 rounded-full -mr-16 -mt-16 transform transition-transform group-hover:scale-150 duration-700"></div>
                            <h3 className="text-blue-400 font-bold text-xs uppercase tracking-widest mb-4">Account Integrity</h3>
                            <div className="space-y-4">
                                <div className="flex items-center justify-between">
                                    <span className="text-gray-400 text-sm">Security Level</span>
                                    <span className="text-yellow-400 font-bold">Standard</span>
                                </div>
                                <div className="w-full bg-gray-800 rounded-full h-2">
                                    <div className="bg-blue-500 h-2 rounded-full" style={{ width: '60%' }}></div>
                                </div>
                                <p className="text-xs text-gray-400 mt-2">Add two-factor authentication to reach 100% security score.</p>
                            </div>
                        </div>
                    </div>

                    {/* Right Column: Detailed Info */}
                    <div className="lg:col-span-2 space-y-8">
                        <section className="bg-white rounded-3xl border border-gray-100 shadow-sm overflow-hidden">
                            <div className="px-8 py-6 border-b border-gray-50 flex justify-between items-center">
                                <h3 className="text-xl font-bold text-gray-900">Personal Information</h3>
                                <button className="text-blue-600 font-bold text-sm hover:text-blue-700 underline-offset-4 hover:underline">Edit</button>
                            </div>
                            <div className="p-8">
                                <dl className="grid grid-cols-1 sm:grid-cols-2 gap-x-8 gap-y-10">
                                    <div>
                                        <dt className="text-sm font-bold text-gray-400 uppercase tracking-wider mb-2">Username</dt>
                                        <dd className="text-gray-900 font-semibold">{user.username || <span className="text-gray-400 italic font-normal">Not set</span>}</dd>
                                    </div>
                                    <div>
                                        <dt className="text-sm font-bold text-gray-400 uppercase tracking-wider mb-2">Email Address</dt>
                                        <dd className="text-gray-900 font-semibold break-all">{user.email}</dd>
                                    </div>
                                    <div>
                                        <dt className="text-sm font-bold text-gray-400 uppercase tracking-wider mb-2">Account ID</dt>
                                        <dd className="font-mono text-sm text-gray-500 bg-gray-50 p-2.5 rounded-xl border border-gray-100 break-all">{user.id}</dd>
                                    </div>
                                    <div>
                                        <dt className="text-sm font-bold text-gray-400 uppercase tracking-wider mb-2">Member Since</dt>
                                        <dd className="text-gray-900 font-semibold">{formatDate(user.created_at)}</dd>
                                    </div>
                                </dl>
                            </div>
                        </section>

                        <section className="bg-white rounded-3xl border border-gray-100 shadow-sm overflow-hidden">
                            <div className="px-8 py-6 border-b border-gray-50">
                                <h3 className="text-xl font-bold text-gray-900">Security Settings</h3>
                            </div>
                            <div className="p-8 space-y-6">
                                <div className="flex items-center justify-between p-4 rounded-2xl bg-gray-50 border border-gray-100 group transition-all hover:bg-white hover:shadow-md">
                                    <div className="flex items-center space-x-4">
                                        <div className="w-12 h-12 bg-white rounded-xl shadow-sm flex items-center justify-center text-xl">
                                            🔑
                                        </div>
                                        <div>
                                            <p className="font-bold text-gray-900">Password</p>
                                            <p className="text-xs text-gray-500">Last changed 3 months ago</p>
                                        </div>
                                    </div>
                                    <button className="px-4 py-2 bg-white text-blue-600 rounded-xl text-sm font-bold border border-blue-100 hover:bg-blue-50 transition-colors shadow-sm">
                                        Change
                                    </button>
                                </div>

                                <div className="flex items-center justify-between p-4 rounded-2xl bg-gray-50 border border-gray-100 group transition-all hover:bg-white hover:shadow-md">
                                    <div className="flex items-center space-x-4">
                                        <div className="w-12 h-12 bg-white rounded-xl shadow-sm flex items-center justify-center text-xl">
                                            📱
                                        </div>
                                        <div>
                                            <p className="font-bold text-gray-900">Two-Factor Auth</p>
                                            <p className="text-xs text-red-500 font-medium">Currently Disabled</p>
                                        </div>
                                    </div>
                                    <button className="px-4 py-2 bg-blue-600 text-white rounded-xl text-sm font-bold hover:bg-blue-700 transition-all shadow-lg shadow-blue-200">
                                        Enable
                                    </button>
                                </div>
                            </div>
                        </section>

                        <div className="pt-4 flex flex-col sm:flex-row gap-4">
                            <button
                                onClick={logout}
                                className="px-10 py-4 bg-red-50 text-red-600 rounded-2xl font-bold hover:bg-red-100 transition-all flex-1 text-center shadow-sm"
                            >
                                Sign Out Everywhere
                            </button>
                        </div>
                    </div>
                </div>
            </div>
        </ErrorBoundary>
    );
};

export default Profile;
