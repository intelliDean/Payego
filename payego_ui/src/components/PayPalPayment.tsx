import React, { useState } from 'react';
import { PayPalButtons } from '@paypal/react-paypal-js';
import { useQueryClient } from '@tanstack/react-query';
import client from '../api/client';

interface PayPalPaymentProps {
    paymentId: string;
    transactionId: string;
    currency: string;
    amount: number;
}

const PayPalPayment: React.FC<PayPalPaymentProps> = ({ paymentId, transactionId, currency, amount }) => {
    const queryClient = useQueryClient();
    const [error, setError] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);

    const getErrorMessage = (message: string) => {
        if (message.includes('INSTRUMENT_DECLINED')) {
            return 'Your payment method was declined. Try a different card!';
        }
        return message || 'PayPal payment failed.';
    };

    return (
        <div className="mt-6">
            <div className="text-center mb-6">
                <div className="w-12 h-12 bg-gradient-to-r from-purple-500 to-pink-500 rounded-xl flex items-center justify-center mx-auto mb-3">
                    <span className="text-white font-bold text-sm">PP</span>
                </div>
                <h3 className="text-xl font-bold text-gray-800 mb-1">Pay with PayPal</h3>
                <p className="text-gray-600 text-sm">Amount: {amount} {currency}</p>
            </div>

            <PayPalButtons
                createOrder={() => Promise.resolve(paymentId)}
                onApprove={async (data) => {
                    setLoading(true);
                    setError(null);
                    try {
                        const response = await client.post('/api/paypal/capture', {
                            order_id: data.orderID,
                            transaction_id: transactionId
                        });

                        if (response.data.status?.toLowerCase() === 'completed') {
                            // Invalidate queries to refresh balance and history
                            queryClient.invalidateQueries({ queryKey: ['wallets'] });
                            queryClient.invalidateQueries({ queryKey: ['transactions'] });

                            window.location.href = `/success?transaction_id=${transactionId}`;
                        } else {
                            setError(getErrorMessage(response.data.error_message));
                        }
                    } catch (err: any) {
                        setError(getErrorMessage(err.response?.data?.error_message || 'Capture failed.'));
                    } finally {
                        setLoading(false);
                    }
                }}
                onError={(err) => {
                    console.error('PayPal error:', err);
                    setError('PayPal checkout failed.');
                }}
                style={{ layout: 'vertical', color: 'blue', label: 'pay' }}
                disabled={loading}
            />

            {error && (
                <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg text-center">
                    <p className="text-red-600 text-sm">{error}</p>
                </div>
            )}

            {loading && (
                <div className="mt-4 p-3 bg-purple-50 border border-purple-200 rounded-lg text-center">
                    <p className="text-purple-600 animate-pulse text-sm">Processing...</p>
                </div>
            )}
        </div>
    );
};

export default PayPalPayment;
