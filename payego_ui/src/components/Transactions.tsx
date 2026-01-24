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

    const getStatusColor = (status: string) => {
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
            case 'InternalTransfer': return '🤝';
            case 'Withdrawal': return '🏦';
            case 'CurrencyConversion': return '🔄';
            default: return '📜';
        }
    };

    return (
        <ErrorBoundary>
            <div className="max-w-4xl mx-auto">
                <div className="mb-8">
                    <h1 className="text-3xl font-bold text-gray-900">Transactions</h1>
                    <p className="text-gray-500">History of your activities</p>
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
                    <div className="bg-white rounded-2xl shadow-sm border border-gray-100 overflow-hidden">
                        <div className="overflow-x-auto">
                            <table className="w-full text-left">
                                <thead className="bg-gray-50 border-b border-gray-100">
                                    <tr>
                                        <th className="px-6 py-4 text-xs font-bold text-gray-500 uppercase tracking-wider">Type</th>
                                        <th className="px-6 py-4 text-xs font-bold text-gray-500 uppercase tracking-wider">Date</th>
                                        <th className="px-6 py-4 text-xs font-bold text-gray-500 uppercase tracking-wider">Amount</th>
                                        <th className="px-6 py-4 text-xs font-bold text-gray-500 uppercase tracking-wider">Status</th>
                                        <th className="px-6 py-4 text-xs font-bold text-gray-500 uppercase tracking-wider">Reference</th>
                                    </tr>
                                </thead>
                                <tbody className="divide-y divide-gray-100">
                                    {transactions?.map((tx) => (
                                        <tr
                                            key={tx.id}
                                            className="hover:bg-gray-50 transition-colors cursor-pointer"
                                            onClick={() => setSelectedTxId(tx.id)}
                                        >
                                            <td className="px-6 py-4 whitespace-nowrap">
                                                <div className="flex items-center space-x-3">
                                                    <div className="w-8 h-8 rounded-lg bg-gray-100 flex items-center justify-center text-lg">
                                                        {getIntentIcon(tx.intent)}
                                                    </div>
                                                    <span className="font-medium text-gray-900">
                                                        {tx.intent.replace(/([A-Z])/g, ' $1').trim()}
                                                    </span>
                                                </div>
                                            </td>
                                            <td className="px-6 py-4 whitespace-nowrap text-sm text-gray-600">
                                                {formatDate(tx.created_at)}
                                            </td>
                                            <td className="px-6 py-4 whitespace-nowrap">
                                                <span className={`font-bold ${tx.amount >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                                                    {formatBalance(tx.amount, tx.currency)}
                                                </span>
                                            </td>
                                            <td className="px-6 py-4 whitespace-nowrap">
                                                <span className={`px-2.5 py-1 rounded-full text-xs font-bold uppercase ${getStatusColor(tx.status)}`}>
                                                    {tx.status}
                                                </span>
                                            </td>
                                            <td className="px-6 py-4 whitespace-nowrap text-sm font-mono text-gray-400">
                                                {tx.reference.substring(0, 8)}...
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
                        <div className="bg-white rounded-2xl shadow-2xl p-8 max-w-md w-full animate-in fade-in zoom-in duration-200">
                            <h3 className="text-2xl font-bold text-gray-900 mb-6">Transaction Details</h3>
                            <div className="space-y-4">
                                <div className="flex justify-between items-center py-2 border-b border-gray-50">
                                    <span className="text-gray-500">Status</span>
                                    <span className={`px-2.5 py-1 rounded-full text-xs font-bold uppercase ${getStatusColor(selectedTransaction.status)}`}>
                                        {selectedTransaction.status}
                                    </span>
                                </div>
                                <div className="flex justify-between items-center py-2 border-b border-gray-50">
                                    <span className="text-gray-500">Type</span>
                                    <span className="font-semibold text-gray-900">{selectedTransaction.intent.replace(/([A-Z])/g, ' $1').trim()}</span>
                                </div>
                                <div className="flex justify-between items-center py-2 border-b border-gray-50">
                                    <span className="text-gray-500">Amount</span>
                                    <span className={`font-bold text-lg ${selectedTransaction.amount >= 0 ? 'text-green-600' : 'text-red-600'}`}>
                                        {formatBalance(selectedTransaction.amount, selectedTransaction.currency)}
                                    </span>
                                </div>
                                <div className="flex justify-between items-center py-2 border-b border-gray-50">
                                    <span className="text-gray-500">Date</span>
                                    <span className="text-gray-900">{formatDate(selectedTransaction.created_at)}</span>
                                </div>
                                <div className="flex justify-between items-center py-2 border-b border-gray-50">
                                    <span className="text-gray-500">Reference</span>
                                    <span className="font-mono text-sm text-gray-900">{selectedTransaction.reference}</span>
                                </div>
                                <div className="flex justify-between items-center py-2 border-b border-gray-50">
                                    <span className="text-gray-500">ID</span>
                                    <span className="font-mono text-xs text-gray-400">{selectedTransaction.id}</span>
                                </div>
                            </div>
                            <button
                                onClick={() => setSelectedTxId(null)}
                                className="mt-8 w-full py-3 bg-gray-100 text-gray-700 rounded-xl font-bold hover:bg-gray-200 transition-colors"
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
