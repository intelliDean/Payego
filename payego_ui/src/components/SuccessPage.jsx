import React from 'react';
import { Link, useSearchParams } from 'react-router-dom';

function SuccessPage() {
    const [searchParams] = useSearchParams();
    const transactionId = searchParams.get('transaction_id');

    return (
        <div className="max-w-md mx-auto mt-10 p-8 bg-white rounded-2xl shadow-xl border border-gray-100 text-center">
            <div className="mb-8">
                <div className="w-20 h-20 bg-gradient-to-r from-green-500 to-emerald-500 rounded-full flex items-center justify-center mx-auto mb-4">
                    <span className="text-white text-3xl">✓</span>
                </div>
                <h2 className="text-3xl font-bold text-gray-800 mb-2">Payment Successful!</h2>
                <p className="text-gray-600">Your transaction has been completed successfully</p>
            </div>
            {transactionId && (
                <div className="mb-6 p-4 bg-gray-50 rounded-lg">
                    <p className="text-sm text-gray-600 mb-1">Transaction ID</p>
                    <p className="font-mono text-sm text-gray-800 break-all">{transactionId}</p>
                </div>
            )}
            <Link
                to="/"
                className="inline-block bg-gradient-to-r from-blue-600 to-indigo-600 text-white px-8 py-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
            >
                Return to Dashboard
            </Link>
        </div>
    );
}

export default SuccessPage;