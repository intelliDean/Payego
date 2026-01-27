import React, { useState } from "react";
import { Link } from "react-router-dom";
import ErrorBoundary from "./ErrorBoundary";
import { useAuth } from "../contexts/AuthContext";
import { useWallets } from "../hooks/useWallets";
import { useTransactions, useTransactionDetails } from "../hooks/useTransactions";


const Dashboard: React.FC = () => {
    const { user } = useAuth();
    const { data: wallets, isLoading: walletsLoading, error: walletsError } = useWallets();
    const { data: transactions, isLoading: transactionsLoading, error: transactionsError } = useTransactions();

    const [selectedTxId, setSelectedTxId] = useState<string | null>(null);
    const { data: selectedTransaction } = useTransactionDetails(selectedTxId || "");

    const formatBalance = (amount: number, currency?: string) => {
        return new Intl.NumberFormat("en-US", {
            style: "currency",
            currency: currency || "USD",
            minimumFractionDigits: 2,
        }).format(amount / 100);
    };

    const formatDate = (dateStr: string) => {
        return new Date(dateStr).toLocaleDateString("en-US", {
            month: "short",
            day: "numeric",
            year: "numeric",
            hour: "2-digit",
            minute: "2-digit",
        });
    };

    const getIntentIcon = (intent: string) => {
        switch (intent) {
            case 'TopUp': return '💰';
            case 'ExternalTransfer': return '💸';
            case 'InternalTransfer':
            case 'Transfer': return '🤝';
            case 'Withdrawal':
            case 'Payout': return '🏦';
            case 'CurrencyConversion':
            case 'Conversion': return '🔄';
            default: return '📜';
        }
    };

    const getIntentLabel = (intent: string) => {
        if (intent === 'Payout') return 'Withdrawal';
        return intent.replace(/([A-Z])/g, ' $1').trim();
    };

    const isLoading = walletsLoading || transactionsLoading;
    const error = walletsError || transactionsError;

    const balancesByCurrency = wallets?.reduce((acc, wallet) => {
        acc[wallet.currency] = (acc[wallet.currency] || 0) + wallet.balance;
        return acc;
    }, {} as Record<string, number>) || {};

    return (
        <ErrorBoundary>
            <div className="min-h-screen">
                {/* Main Content */}
                <div className="p-4 md:p-6">
                    <div className="max-w-5xl mx-auto">
                        <div className="flex justify-between items-center mb-10">
                            <div>
                                <h1 className="text-3xl font-black text-gray-900 tracking-tight">Dashboard</h1>
                                <p className="text-base text-gray-500 mt-1">Welcome back, your financial snapshot is ready.</p>
                            </div>
                            {user && (
                                <Link
                                    to="/profile"
                                    className="flex items-center space-x-3 bg-white rounded-2xl px-4 py-2 shadow-sm border border-gray-100 hover:shadow-md transition-all duration-200"
                                >
                                    <div className="w-10 h-10 bg-gradient-to-br from-blue-600 to-indigo-600 rounded-xl flex items-center justify-center shadow-lg">
                                        <span className="text-white font-bold">
                                            {user.email?.charAt(0)?.toUpperCase()}
                                        </span>
                                    </div>
                                    <div className="text-left hidden sm:block">
                                        <p className="text-sm font-bold text-gray-900 leading-tight">{user.username || 'User'}</p>
                                        <p className="text-xs text-gray-500">{user.email}</p>
                                    </div>
                                </Link>
                            )}
                        </div>

                        {isLoading && (
                            <div className="flex flex-col items-center justify-center h-64">
                                <div className="animate-spin rounded-full h-10 w-10 border-b-2 border-blue-600"></div>
                                <p className="mt-3 text-gray-600">Fetching your dashboard...</p>
                            </div>
                        )}

                        {error && (
                            <div className="rounded-lg bg-red-50 p-3 mb-4">
                                <div className="flex">
                                    <div className="flex-shrink-0">
                                        <svg className="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor">
                                            <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z" clipRule="evenodd" />
                                        </svg>
                                    </div>
                                    <div className="ml-3">
                                        <p className="text-sm font-medium text-red-800">Something went wrong. Please try again.</p>
                                    </div>
                                </div>
                            </div>
                        )}

                        {user && !isLoading && (
                            <div className="space-y-4">
                                {/* Stats */}
                                <div className="bg-white rounded-lg shadow-sm border border-gray-100">
                                    <div className="p-4">
                                        <h3 className="text-lg font-semibold text-gray-900 mb-3">Balances</h3>
                                        {Object.keys(balancesByCurrency).length > 0 ? (
                                            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                                {Object.entries(balancesByCurrency).map(([currency, balance]) => (
                                                    <div
                                                        key={currency}
                                                        className="bg-gradient-to-r from-blue-50 to-indigo-50 rounded-lg p-3 border border-blue-100"
                                                    >
                                                        <h4 className="text-sm font-medium text-gray-500">{currency}</h4>
                                                        <p className="text-lg font-bold text-gray-900">
                                                            {formatBalance(balance, currency)}
                                                        </p>
                                                    </div>
                                                ))}
                                            </div>
                                        ) : (
                                            <div className="text-center py-4 bg-gray-50 rounded-b-lg">
                                                <div className="inline-flex items-center justify-center w-10 h-10 bg-gray-100 rounded-full mb-2">
                                                    <span className="text-lg">💳</span>
                                                </div>
                                                <p className="text-gray-600 text-sm">No wallets yet!</p>
                                            </div>
                                        )}
                                    </div>
                                </div>

                                {/* Wallets */}
                                <div className="bg-white rounded-lg shadow-sm border border-gray-100">
                                    <div className="p-4 flex justify-between items-center">
                                        <h2 className="text-lg font-semibold text-gray-900">Wallets</h2>
                                        <Link to="/wallets" className="text-sm text-blue-600 hover:text-blue-700">
                                            View All
                                        </Link>
                                    </div>
                                    {wallets && wallets.length > 0 ? (
                                        <div className="grid grid-cols-1 gap-3 p-4 pt-0">
                                            {wallets.slice(0, 3).map((wallet) => (
                                                <div
                                                    key={wallet.id}
                                                    className="bg-gradient-to-r from-blue-50 to-indigo-50 rounded-lg p-3 border border-blue-100 hover:from-blue-100 hover:to-indigo-100 transition-all duration-200"
                                                >
                                                    <div className="flex items-center justify-between">
                                                        <div className="flex items-center space-x-2">
                                                            <div className="w-8 h-8 bg-blue-100 rounded-lg flex items-center justify-center">
                                                                <span className="text-blue-600 font-medium text-sm">
                                                                    {wallet.currency}
                                                                </span>
                                                            </div>
                                                            <span className="text-sm font-medium text-gray-500">Balance</span>
                                                        </div>
                                                        <p className="text-base font-bold text-gray-900">
                                                            {formatBalance(wallet.balance, wallet.currency)}
                                                        </p>
                                                    </div>
                                                </div>
                                            ))}
                                        </div>
                                    ) : (
                                        <div className="text-center py-4 bg-gray-50 rounded-b-lg">
                                            <div className="inline-flex items-center justify-center w-10 h-10 bg-gray-100 rounded-full mb-2">
                                                <span className="text-lg">💳</span>
                                            </div>
                                            <p className="text-gray-600 text-sm">No wallets yet!</p>
                                        </div>
                                    )}
                                </div>

                                {/* Transactions */}
                                <div className="bg-white rounded-lg shadow-sm border border-gray-100">
                                    <div className="p-4 flex justify-between items-center">
                                        <h2 className="text-lg font-semibold text-gray-900">Recent Transactions</h2>
                                        <Link to="/transactions" className="text-sm text-blue-600 hover:text-blue-700">
                                            View All
                                        </Link>
                                    </div>
                                    {transactions && transactions.length > 0 ? (
                                        <div className="divide-y divide-gray-200">
                                            {transactions.slice(0, 5).map((tx) => (
                                                <button
                                                    key={tx.id}
                                                    onClick={() => setSelectedTxId(tx.id)}
                                                    className="w-full flex items-center justify-between p-3 hover:bg-gray-50 transition-all duration-200"
                                                >
                                                    <div className="flex items-center space-x-2">
                                                        <div className="w-6 h-6 bg-gray-100 rounded-lg flex items-center justify-center">
                                                            <span className="text-sm">
                                                                {getIntentIcon(tx.intent)}
                                                            </span>
                                                        </div>
                                                        <div className="text-left">
                                                            <p className="text-sm font-medium text-gray-900 capitalize">{getIntentLabel(tx.intent)}</p>
                                                            <p className="text-xs text-gray-500">{formatDate(tx.created_at)}</p>
                                                        </div>
                                                    </div>
                                                    <div className="text-right">
                                                        <p className={`text-sm font-medium ${tx.amount >= 0 ? "text-green-600" : "text-red-600"}`}>
                                                            {formatBalance(tx.amount, tx.currency)}
                                                        </p>
                                                        <p className="text-xs text-gray-500 capitalize">{tx.status || 'Pending'}</p>
                                                    </div>
                                                </button>
                                            ))}
                                        </div>
                                    ) : (
                                        <div className="text-center py-4 bg-gray-50 rounded-b-lg">
                                            <div className="inline-flex items-center justify-center w-10 h-10 bg-gray-100 rounded-full mb-2">
                                                <span className="text-lg">📜</span>
                                            </div>
                                            <p className="text-gray-600 text-sm">No transactions yet!</p>
                                        </div>
                                    )}
                                </div>

                                {/* Quick Actions */}
                                <div className="bg-white rounded-lg shadow-sm border border-gray-100">
                                    <div className="p-4">
                                        <h2 className="text-lg font-semibold text-gray-900 mb-3">Quick Actions</h2>
                                        <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                                            {[
                                                { to: "/top-up", label: "Top Up", icon: "💰", gradient: "from-green-50 to-emerald-50", border: "border-green-100 hover:border-green-200" },
                                                { to: "/transfer", label: "Transfer", icon: "💸", gradient: "from-blue-50 to-indigo-50", border: "border-blue-100 hover:border-blue-200" },
                                                { to: "/withdraw", label: "Withdraw", icon: "🏦", gradient: "from-purple-50 to-pink-50", border: "border-purple-100 hover:border-purple-200" },
                                                { to: "/convert", label: "Convert", icon: "🔄", gradient: "from-blue-50 to-indigo-50", border: "border-blue-100 hover:border-blue-200" },
                                            ].map((action) => (
                                                <Link
                                                    key={action.to}
                                                    to={action.to}
                                                    className={`group relative bg-gradient-to-r ${action.gradient} rounded-lg p-3 border ${action.border} transition-all duration-200 overflow-hidden`}
                                                >
                                                    <div className="relative z-10 flex items-center space-x-2">
                                                        <div className="w-6 h-6 bg-opacity-50 bg-white rounded-lg flex items-center justify-center">
                                                            <span className="text-base">{action.icon}</span>
                                                        </div>
                                                        <h3 className="text-sm font-semibold text-gray-900">{action.label}</h3>
                                                    </div>
                                                    <div className="absolute bottom-0 right-0 w-12 h-12 bg-gradient-to-r from-white to-transparent rounded-full -mr-6 -mb-6 transform group-hover:scale-110 transition-transform duration-200 opacity-50"></div>
                                                </Link>
                                            ))}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        )}

                        {/* Transaction Details Modal */}
                        {selectedTransaction && (
                            <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
                                <div className="bg-white rounded-lg p-6 max-w-sm w-full">
                                    <h3 className="text-lg font-semibold text-gray-900 mb-4">Transaction Details</h3>
                                    <div className="space-y-3">
                                        <p className="text-sm text-gray-600"><span className="font-medium">ID:</span> {selectedTransaction.id}</p>
                                        <p className="text-sm text-gray-600"><span className="font-medium">Type:</span> {getIntentLabel(selectedTransaction.intent).toUpperCase()}</p>
                                        <p className="text-sm text-gray-600"><span className="font-medium">Amount:</span> {formatBalance(selectedTransaction.amount, selectedTransaction.currency)}</p>
                                        <p className="text-sm text-gray-600"><span className="font-medium">Date:</span> {formatDate(selectedTransaction.created_at)}</p>
                                        <p className="text-sm text-gray-600"><span className="font-medium">Status:</span> {(selectedTransaction.status || 'Pending').toUpperCase()}</p>
                                    </div>
                                    <button
                                        onClick={() => setSelectedTxId(null)}
                                        className="mt-4 w-full bg-gray-200 text-gray-700 p-2 rounded-lg hover:bg-gray-300 transition-all duration-200"
                                    >
                                        Close
                                    </button>
                                </div>
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </ErrorBoundary>
    );
}

export default Dashboard;
