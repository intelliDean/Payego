import React from 'react';
import { useWallets } from '../hooks/useWallets';
import ErrorBoundary from './ErrorBoundary';
import { useNavigate } from 'react-router-dom';

const Wallets: React.FC = () => {
    const { data: wallets, isLoading, error } = useWallets();
    const navigate = useNavigate();

    const formatBalance = (amount: number, currency: string) => {
        return new Intl.NumberFormat('en-US', {
            style: 'currency',
            currency: currency,
            minimumFractionDigits: 2,
        }).format(amount / 100);
    };

    return (
        <ErrorBoundary>
            <div className="max-w-4xl mx-auto">
                <div className="flex justify-between items-center mb-8">
                    <div>
                        <h1 className="text-3xl font-bold text-gray-900">Your Wallets</h1>
                        <p className="text-gray-500">Manage your multi-currency balances</p>
                    </div>
                    <button
                        onClick={() => navigate('/top-up')}
                        className="btn-primary flex items-center space-x-2"
                    >
                        <span>Add Funds</span>
                    </button>
                </div>

                {isLoading ? (
                    <div className="flex flex-col items-center justify-center h-64">
                        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600"></div>
                        <p className="mt-4 text-gray-600">Loading your wallets...</p>
                    </div>
                ) : error ? (
                    <div className="p-4 bg-red-50 border border-red-200 rounded-xl text-red-600 text-center">
                        Failed to load wallets. Please try again.
                    </div>
                ) : (
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                        {wallets?.map((wallet) => (
                            <div
                                key={wallet.id}
                                className="bg-white rounded-2xl shadow-sm border border-gray-100 p-6 hover:shadow-md transition-shadow duration-200"
                            >
                                <div className="flex justify-between items-start mb-4">
                                    <div className="w-12 h-12 bg-gradient-to-br from-purple-500 to-indigo-500 rounded-xl flex items-center justify-center text-white font-bold text-lg">
                                        {wallet.currency}
                                    </div>
                                    <span className="px-3 py-1 bg-green-50 text-green-600 text-xs font-bold rounded-full uppercase">
                                        Active
                                    </span>
                                </div>
                                <div className="space-y-1">
                                    <p className="text-sm text-gray-500 font-medium">Available Balance</p>
                                    <h3 className="text-2xl font-black text-gray-900">
                                        {formatBalance(wallet.balance, wallet.currency)}
                                    </h3>
                                </div>
                                <div className="mt-6 pt-6 border-t border-gray-50 flex space-x-3">
                                    <button
                                        onClick={() => navigate('/transfer', { state: { currency: wallet.currency } })}
                                        className="flex-1 py-2 px-4 bg-gray-50 text-gray-700 rounded-lg text-sm font-semibold hover:bg-gray-100 transition-colors"
                                    >
                                        Transfer
                                    </button>
                                    <button
                                        onClick={() => navigate('/convert', { state: { fromCurrency: wallet.currency } })}
                                        className="flex-1 py-2 px-4 bg-gray-50 text-gray-700 rounded-lg text-sm font-semibold hover:bg-gray-100 transition-colors"
                                    >
                                        Convert
                                    </button>
                                </div>
                            </div>
                        ))}

                        {wallets?.length === 0 && (
                            <div className="col-span-full bg-gray-50 rounded-2xl p-12 text-center border-2 border-dashed border-gray-200">
                                <div className="w-16 h-16 bg-white rounded-full flex items-center justify-center mx-auto mb-4 shadow-sm text-2xl">
                                    💳
                                </div>
                                <h3 className="text-lg font-bold text-gray-900">No wallets yet</h3>
                                <p className="text-gray-500 mb-6">Create your first wallet by adding funds.</p>
                                <button
                                    onClick={() => navigate('/top-up')}
                                    className="btn-primary"
                                >
                                    Get Started
                                </button>
                            </div>
                        )}
                    </div>
                )}
            </div>
        </ErrorBoundary>
    );
};

export default Wallets;
