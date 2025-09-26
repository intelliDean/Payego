import React from 'react';
import { Link, useSearchParams } from 'react-router-dom';

function SuccessPage() {
    const [searchParams] = useSearchParams();
    const transactionId = searchParams.get('transaction_id');

    return (
        <div className="max-w-md mx-auto mt-10 p-6 bg-white rounded-lg shadow-md text-center">
            <h2 className="text-2xl font-bold mb-6 text-green-600">Payment Successful</h2>
            <p className="text-gray-700 mb-4">
                Your payment has been processed successfully.
            </p>
            {transactionId && (
                <p className="text-gray-700 mb-4">
                    <strong>Transaction ID:</strong> {transactionId}
                </p>
            )}
            <Link
                to="/"
                className="bg-blue-600 text-white p-2 rounded hover:bg-blue-700"
            >
                Return to Dashboard
            </Link>
        </div>
    );
}

export default SuccessPage;