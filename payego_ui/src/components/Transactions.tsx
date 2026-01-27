import React, { useState } from 'react';
import { useTransactions, useTransactionDetails } from '../hooks/useTransactions';
import ErrorBoundary from './ErrorBoundary';

const Transactions: React.FC = () => {
    const { data: transactions, isLoading, error } = useTransactions();
    const [selectedTxId, setSelectedTxId] = useState<string | null>(null);
    const { data: selectedTransaction } = useTransactionDetails(selectedTxId || "");

    const formatBalance = (amount: number, currency: string) => {
        return new Intl.NumberFormat('en-US', {
            style: 'currency',
            currency: currency,
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

    const getStatusColor = (status?: string) => {
        if (!status) return 'text-gray-600 bg-gray-50';
        switch (status.toLowerCase()) {
            case 'completed': return 'text-green-600 bg-green-50';
            case 'pending': return 'text-amber-600 bg-amber-50';
            case 'failed': return 'text-red-600 bg-red-50';
            default: return 'text-gray-600 bg-gray-50';
        }
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

    return (
        <ErrorBoundary>
            <div className="max-w-4xl mx-auto">
                <div className="mb-8">
                    <h1 className="text-3xl font-bold text-main">Transactions</h1>
                    <p className="text-muted">History of your activities</p>
                </div>

                {isLoading ? (
                    <div className="flex flex-col items-center justify-center h-64">
                        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600"></div>
                        <p className="mt-4 text-gray-600">Loading transactions...</p>
                    </div>
                ) : error ? (
                    <div className="p-4 bg-red-50 border border-red-200 rounded-xl text-red-600 text-center">
                        Failed to load transactions. Please try again.
                    </div>
                ) : (
                    <div className="bg-card rounded-2xl shadow-sm border border-main overflow-hidden">
                        <div className="overflow-x-auto">
                            <table className="w-full text-left">
                                <thead className="bg-main border-b border-main">
                                    <tr>
                                        <th className="px-6 py-4 text-xs font-bold text-muted uppercase tracking-wider">Type</th>
                                        <th className="px-6 py-4 text-xs font-bold text-muted uppercase tracking-wider">Date</th>
                                        <th className="px-6 py-4 text-xs font-bold text-muted uppercase tracking-wider">Amount</th>
                                        <th className="px-6 py-4 text-xs font-bold text-muted uppercase tracking-wider">Status</th>
                                        <th className="px-6 py-4 text-xs font-bold text-muted uppercase tracking-wider">Reference</th>
                                    </tr>
                                </thead>
                                <tbody className="divide-y divide-main">
                                    {transactions?.map((tx) => (
                                        <tr
                                            key={tx.id}
                                            className="hover:bg-main transition-colors cursor-pointer"
                                            onClick={() => setSelectedTxId(tx.id)}
                                        >
                                            <td className="px-6 py-4 whitespace-nowrap">
                                                <div className="flex items-center space-x-3">
                                                    <div className="w-8 h-8 rounded-lg bg-main flex items-center justify-center text-lg">
                                                        {getIntentIcon(tx.intent)}
                                                    </div>
                                                    <span className="font-medium text-main">
                                                        {getIntentLabel(tx.intent)}
                                                    </span>
                                                </div>
                                            </td>
                                            <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-600">
                                                {formatDate(tx.created_at)}
                                            </td>
                                            <td className="px-6 py-4 whitespace-nowrap">
                                                <span className={`font-bold ${tx.amount >= 0 ? 'text-green-600 dark:text-emerald-400' : 'text-red-600 dark:text-rose-400'}`}>
                                                    {formatBalance(tx.amount, tx.currency)}
                                                </span>
                                            </td>
                                            <td className="px-6 py-4 whitespace-nowrap">
                                                <span className={`px-2.5 py-1 rounded-full text-xs font-bold uppercase ${getStatusColor(tx.status)}`}>
                                                    {tx.status}
                                                </span>
                                            </td>
                                            <td className="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-400">
                                                {tx.reference ? `${tx.reference.substring(0, 8)}...` : 'N/A'}
                                            </td>
                                        </tr>
                                    ))}
                                </tbody>
                            </table>
                        </div>
                        {transactions?.length === 0 && (
                            <div className="p-12 text-center">
                                <p className="text-gray-500">No transactions found.</p>
                            </div>
                        )}
                    </div>
                )}

                {/* Transaction Details Modal */}
                {selectedTransaction && (
                    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-50 p-4">
                        <div className="bg-card rounded-2xl shadow-2xl p-8 max-w-md w-full animate-in fade-in zoom-in duration-200 border border-main">
                            <h3 className="text-2xl font-bold text-main mb-6">Transaction Details</h3>
                            <div className="space-y-4">
                                <div className="flex justify-between items-center py-2 border-b border-main">
                                    <span className="text-muted">Status</span>
                                    <span className={`px-2.5 py-1 rounded-full text-xs font-bold uppercase ${getStatusColor(selectedTransaction.status)}`}>
                                        {selectedTransaction.status}
                                    </span>
                                </div>
                                <div className="flex justify-between items-center py-2 border-b border-main">
                                    <span className="text-muted">Type</span>
                                    <span className="font-semibold text-main">{getIntentLabel(selectedTransaction.intent)}</span>
                                </div>
                                <div className="flex justify-between items-center py-2 border-b border-gray-50 dark:border-slate-800">
                                    <span className="text-gray-500 dark:text-slate-400">Amount</span>
                                    <span className={`font-bold text-lg ${selectedTransaction.amount >= 0 ? 'text-green-600 dark:text-emerald-400' : 'text-red-600 dark:text-rose-400'}`}>
                                        {formatBalance(selectedTransaction.amount, selectedTransaction.currency)}
                                    </span>
                                </div>
                                <div className="flex justify-between items-center py-2 border-b border-main">
                                    <span className="text-muted">Date</span>
                                    <span className="text-main">{formatDate(selectedTransaction.created_at)}</span>
                                </div>
                                <div className="flex justify-between items-center py-2 border-b border-main">
                                    <span className="text-muted">Reference</span>
                                    <span className="font-mono text-sm text-main">{selectedTransaction.reference}</span>
                                </div>
                                <div className="flex justify-between items-center py-2 border-b border-main">
                                    <span className="text-muted">ID</span>
                                    <span className="font-mono text-xs text-muted">{selectedTransaction.id}</span>
                                </div>
                            </div>
                            <button
                                onClick={() => setSelectedTxId(null)}
                                className="mt-8 w-full py-3 bg-main text-main rounded-xl font-bold hover:opacity-80 transition-colors"
                            >
                                Close
                            </button>
                        </div>
                    </div>
                )}
            </div>
        </ErrorBoundary>
    );
};

export default Transactions;
