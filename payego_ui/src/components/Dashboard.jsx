import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { Link } from 'react-router-dom';

function Dashboard() {
    const [user, setUser] = useState(null);
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchUser = async () => {
            try {
                const response = await axios.get(`${import.meta.env.VITE_API_URL}/api/current_user`, {
                    headers: {
                        'Authorization': `Bearer ${localStorage.getItem('jwt_token')}`,
                    },
                });
                setUser(response.data);
            } catch (err) {
                setError(err.response?.data?.message || 'Failed to fetch user data');
            } finally {
                setLoading(false);
            }
        };
        fetchUser();
    }, []);

    // Format balance based on currency
    const formatBalance = (balance, currency) => {
        return new Intl.NumberFormat('en-US', {
            style: 'currency',
            currency: currency,
            minimumFractionDigits: 2,
        }).format(balance / 100);
    };

    return (
        <div className="max-w-4xl mx-auto">
            <div className="mb-8">
                <h1 className="text-4xl font-bold text-gray-800 mb-2">Dashboard</h1>
                <p className="text-gray-600">Manage your finances with ease</p>
            </div>
            
            {loading && (
                <div className="flex justify-center items-center py-12">
                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                </div>
            )}
            
            {error && (
                <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
                    <p className="text-red-600 text-center">{error}</p>
                </div>
            )}
            
            {user && (
                <div className="space-y-8">
                    {/* User Info Card */}
                    <div className="bg-white rounded-2xl shadow-lg p-6 border border-gray-100">
                        <div className="flex items-center space-x-4 mb-6">
                            <div className="w-12 h-12 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-full flex items-center justify-center">
                                <span className="text-white font-bold text-lg">{user.email.charAt(0).toUpperCase()}</span>
                            </div>
                            <div>
                                <h2 className="text-xl font-semibold text-gray-800">Welcome back!</h2>
                                <p className="text-gray-600">{user.email}</p>
                            </div>
                        </div>
                    </div>

                    {/* Wallet Balances */}
                    <div className="bg-white rounded-2xl shadow-lg p-6 border border-gray-100">
                        <h3 className="text-2xl font-bold text-gray-800 mb-6">Wallet Balances</h3>
                        {user.wallets && user.wallets.length > 0 ? (
                            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                                {user.wallets.map((wallet) => (
                                    <div key={wallet.currency} className="bg-gradient-to-r from-blue-50 to-indigo-50 rounded-xl p-4 border border-blue-100">
                                        <div className="flex items-center justify-between">
                                            <div>
                                                <p className="text-sm font-medium text-gray-600">{wallet.currency}</p>
                                                <p className="text-2xl font-bold text-gray-800">
                                                    {formatBalance(wallet.balance, wallet.currency)}
                                                </p>
                                            </div>
                                            <div className="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
                                                <span className="text-blue-600 font-bold text-sm">{wallet.currency}</span>
                                            </div>
                                        </div>
                                    </div>
                                ))}
                            </div>
                        ) : (
                            <div className="text-center py-8">
                                <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
                                    <span className="text-gray-400 text-2xl">💳</span>
                                </div>
                                <p className="text-gray-600">No wallets found. Start by adding funds to your account.</p>
                            </div>
                        )}
                    </div>

                    {/* Quick Actions */}
                    <div className="bg-white rounded-2xl shadow-lg p-6 border border-gray-100">
                        <h3 className="text-2xl font-bold text-gray-800 mb-6">Quick Actions</h3>
                        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
                            <Link 
                                to="/top-up" 
                                className="group bg-gradient-to-r from-green-500 to-emerald-500 text-white p-6 rounded-xl hover:from-green-600 hover:to-emerald-600 transition-all duration-200 shadow-lg hover:shadow-xl transform hover:-translate-y-1"
                            >
                                <div className="flex items-center space-x-3">
                                    <span className="text-2xl">💰</span>
                                    <div>
                                        <h4 className="font-semibold">Top Up</h4>
                                        <p className="text-sm opacity-90">Add funds to your wallet</p>
                                    </div>
                                </div>
                            </Link>
                            
                            <Link 
                                to="/transfer" 
                                className="group bg-gradient-to-r from-blue-500 to-indigo-500 text-white p-6 rounded-xl hover:from-blue-600 hover:to-indigo-600 transition-all duration-200 shadow-lg hover:shadow-xl transform hover:-translate-y-1"
                            >
                                <div className="flex items-center space-x-3">
                                    <span className="text-2xl">💸</span>
                                    <div>
                                        <h4 className="font-semibold">Transfer</h4>
                                        <p className="text-sm opacity-90">Send money to others</p>
                                    </div>
                                </div>
                            </Link>
                            
                            <Link 
                                to="/withdraw" 
                                className="group bg-gradient-to-r from-purple-500 to-pink-500 text-white p-6 rounded-xl hover:from-purple-600 hover:to-pink-600 transition-all duration-200 shadow-lg hover:shadow-xl transform hover:-translate-y-1"
                            >
                                <div className="flex items-center space-x-3">
                                    <span className="text-2xl">🏦</span>
                                    <div>
                                        <h4 className="font-semibold">Withdraw</h4>
                                        <p className="text-sm opacity-90">Transfer to bank account</p>
                                    </div>
                                </div>
                            </Link>
                            
                            <Link 
                                to="/convert" 
                                className="group bg-gradient-to-r from-orange-500 to-red-500 text-white p-6 rounded-xl hover:from-orange-600 hover:to-red-600 transition-all duration-200 shadow-lg hover:shadow-xl transform hover:-translate-y-1"
                            >
                                <div className="flex items-center space-x-3">
                                    <span className="text-2xl">🔄</span>
                                    <div>
                                        <h4 className="font-semibold">Convert</h4>
                                        <p className="text-sm opacity-90">Exchange currencies</p>
                                    </div>
                                </div>
                            </Link>
                            
                            <Link 
                                to="/add-bank" 
                                className="group bg-gradient-to-r from-teal-500 to-cyan-500 text-white p-6 rounded-xl hover:from-teal-600 hover:to-cyan-600 transition-all duration-200 shadow-lg hover:shadow-xl transform hover:-translate-y-1"
                            >
                                <div className="flex items-center space-x-3">
                                    <span className="text-2xl">🏛️</span>
                                    <div>
                                        <h4 className="font-semibold">Manage Banks</h4>
                                        <p className="text-sm opacity-90">Add bank accounts</p>
                                    </div>
                                </div>
                            </Link>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}

export default Dashboard;