import React from "react";
import { Link, useSearchParams } from "react-router-dom";
import ErrorBoundary from "./ErrorBoundary";
import { useTransactionDetails } from "../hooks/useTransactions";

const SuccessPage: React.FC = () => {
    const [searchParams] = useSearchParams();
    // Stripe redirects with 'tx' parameter, not 'transaction_id'
    const transactionId = searchParams.get("tx") || searchParams.get("transaction_id") || "";
    const { data: transaction, isLoading, error } = useTransactionDetails(transactionId);

    const formatAmount = (amount: number, currency?: string) =>
        new Intl.NumberFormat("en-US", {
            style: "currency",
            currency: currency || "USD",
            minimumFractionDigits: 2,
        }).format(amount / 100);

    const formatDate = (dateStr?: string) =>
        dateStr
            ? new Date(dateStr).toLocaleDateString("en-US", {
                month: "short",
                day: "numeric",
                year: "numeric",
                hour: "2-digit",
                minute: "2-digit",
            })
            : "N/A";

    return (
        <ErrorBoundary>
            <div className="min-h-screen bg-gray-50 flex items-center justify-center p-4 sm:p-6">
                <div className="max-w-md mx-auto p-6 sm:p-8 bg-white rounded-2xl shadow-xl border border-gray-100 text-center">
                    {isLoading ? (
                        <div className="flex flex-col items-center justify-center py-8">
                            <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                            <p className="mt-2 text-gray-600 text-sm sm:text-base">
                                Fetching transaction details...
                            </p>
                        </div>
                    ) : error || !transaction ? (
                        <div className="space-y-4">
                            <div className="w-16 sm:w-20 h-16 sm:h-20 bg-gradient-to-r from-red-500 to-pink-500 rounded-full flex items-center justify-center mx-auto mb-4">
                                <svg className="w-8 sm:w-10 h-8 sm:h-10 text-white" viewBox="0 0 24 24" fill="currentColor">
                                    <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z" />
                                </svg>
                            </div>
                            <h2 className="text-xl sm:text-2xl font-semibold text-gray-900 mb-2">
                                Oops!
                            </h2>
                            <p className="text-red-600 text-sm sm:text-base mb-4">
                                {error ? "Transaction details got lost!" : "No transaction found."}
                            </p>
                            <Link
                                to="/"
                                className="inline-block bg-gradient-to-r from-blue-600 to-indigo-600 text-white px-6 sm:px-8 py-3 rounded-lg font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5 text-sm sm:text-base"
                            >
                                Return to Dashboard
                            </Link>
                        </div>
                    ) : (
                        <>
                            <div className="mb-8">
                                <div className="w-16 sm:w-20 h-16 sm:h-20 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-full flex items-center justify-center mx-auto mb-4">
                                    <svg className="w-8 sm:w-10 h-8 sm:h-10 text-white" viewBox="0 0 24 24" fill="currentColor">
                                        <path d="M20.285 2l-11.285 11.567-5.286-5.011-3.714 3.716 9 8.728 15-15.285z" />
                                    </svg>
                                </div>
                                <h2 className="text-2xl sm:text-3xl font-bold text-gray-900 mb-2">
                                    {(transaction.intent === 'Payout' || transaction.intent === 'Withdrawal') ? 'Withdrawal Successful!' :
                                        transaction.intent === 'TopUp' ? 'Top-up Successful!' :
                                            'Transfer Successful!'}
                                </h2>
                                <p className="text-gray-600 text-sm sm:text-base">
                                    {(transaction.intent === 'Payout' || transaction.intent === 'Withdrawal') ? 'Your funds are on the way to your bank account!' :
                                        'Your wallet\'s feeling heavier!'}
                                </p>
                            </div>
                            <div className="mb-6 p-4 bg-gray-50 rounded-lg space-y-2 text-left">
                                <div className="flex flex-col sm:flex-row sm:justify-between gap-1 sm:gap-0">
                                    <p className="text-sm text-gray-600">Transaction ID</p>
                                    <p className="font-mono text-xs sm:text-sm text-gray-800 break-all">{transaction.id}</p>
                                </div>
                                <div className="flex flex-col sm:flex-row sm:justify-between gap-1 sm:gap-0">
                                    <p className="text-sm text-gray-600">Type</p>
                                    <p className="text-sm text-gray-800 capitalize">{transaction.intent.replace(/([A-Z])/g, ' $1').trim()}</p>
                                </div>
                                <div className="flex flex-col sm:flex-row sm:justify-between gap-1 sm:gap-0">
                                    <p className="text-sm text-gray-600">Amount</p>
                                    <p className="text-sm text-gray-800">{formatAmount(transaction.amount, transaction.currency)}</p>
                                </div>
                                <div className="flex flex-col sm:flex-row sm:justify-between gap-1 sm:gap-0">
                                    <p className="text-sm text-gray-600">Date</p>
                                    <p className="text-sm text-gray-800">{formatDate(transaction.created_at)}</p>
                                </div>
                                <div className="flex flex-col sm:flex-row sm:justify-between gap-1 sm:gap-0">
                                    <p className="text-sm text-gray-600">Status</p>
                                    <p className="text-sm text-gray-800 capitalize">{transaction.status}</p>
                                </div>
                            </div>
                            <Link
                                to="/"
                                className="inline-block bg-gradient-to-r from-blue-600 to-indigo-600 text-white px-6 sm:px-8 py-3 rounded-lg font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5 text-sm sm:text-base"
                            >
                                Return to Dashboard
                            </Link>
                        </>
                    )}
                </div>
            </div>
        </ErrorBoundary>
    );
};

export default SuccessPage;
