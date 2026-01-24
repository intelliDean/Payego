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

    return (
        <ErrorBoundary>
            <div className="max-w-2xl mx-auto">
                <div className="mb-8">
                    <h1 className="text-3xl font-bold text-gray-900">Your Profile</h1>
                    <p className="text-gray-500">Manage your account information</p>
                </div>

                <div className="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
                    <div className="p-8 bg-gradient-to-r from-purple-600 to-blue-600 flex items-center space-x-6">
                        <div className="w-24 h-24 bg-white/20 backdrop-blur-md rounded-full border-4 border-white/30 flex items-center justify-center text-4xl text-white font-black shadow-lg">
                            {user.username.charAt(0).toUpperCase()}
                        </div>
                        <div>
                            <h2 className="text-2xl font-bold text-white">{user.username}</h2>
                            <p className="text-white/80 font-medium">{user.email}</p>
                        </div>
                    </div>

                    <div className="p-8 space-y-8">
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-8">
                            <div>
                                <h3 className="text-sm font-bold text-gray-400 uppercase tracking-wider mb-2">Account ID</h3>
                                <p className="font-mono text-gray-900 bg-gray-50 p-3 rounded-lg border border-gray-100">{user.id}</p>
                            </div>
                            <div>
                                <h3 className="text-sm font-bold text-gray-400 uppercase tracking-wider mb-2">Member Since</h3>
                                <p className="text-gray-900 bg-gray-50 p-3 rounded-lg border border-gray-100">{formatDate(user.created_at)}</p>
                            </div>
                        </div>

                        <div className="pt-8 border-t border-gray-100 flex flex-col sm:flex-row gap-4">
                            <button className="btn-primary-glow flex-1 font-bold">
                                Edit Profile
                            </button>
                            <button
                                onClick={logout}
                                className="px-6 py-3 bg-red-50 text-red-600 rounded-xl font-bold hover:bg-red-100 transition-colors flex-1"
                            >
                                Sign Out
                            </button>
                        </div>
                    </div>
                </div>

                <div className="mt-8 p-6 bg-amber-50 rounded-2xl border border-amber-100 flex items-start space-x-4">
                    <div className="w-10 h-10 bg-amber-100 rounded-lg flex items-center justify-center text-xl">
                        🔒
                    </div>
                    <div>
                        <h4 className="text-amber-900 font-bold mb-1">Account Security</h4>
                        <p className="text-amber-800 text-sm leading-relaxed">
                            Your account is protected with enterprise-grade encryption. We never share your sensitive data with third parties.
                        </p>
                    </div>
                </div>
            </div>
        </ErrorBoundary>
    );
};

export default Profile;
