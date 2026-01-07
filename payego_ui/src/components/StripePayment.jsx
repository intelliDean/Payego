import React, { useState } from 'react';
import { CardElement, useStripe, useElements } from '@stripe/react-stripe-js';
function StripePayment({ clientSecret, transactionId, currency }) {
    const stripe = useStripe();
    const elements = useElements();
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(false);
    const [success, setSuccess] = useState(false);

    const handleSubmit = async (event) => {
        event.preventDefault();
        setLoading(true);
        setError(null);

        if (!stripe || !elements) {
            setError('Stripe.js has not loaded.');
            setLoading(false);
            return;
        }

        const cardElement = elements.getElement(CardElement);

        try {
            const result = await stripe.confirmCardPayment(clientSecret, {
                payment_method: {
                    card: cardElement,
                    billing_details: {
                        name: 'Test User', // TODO: Replace with actual user name from /current_user
                    },
                },
                // Removed currency parameter
            });

            if (result.error) {
                setError(result.error.message);
            } else if (result.paymentIntent.status === 'succeeded') {
                setSuccess(true);
                setTimeout(() => {
                    window.location.href = `/success?transaction_id=${transactionId}`;
                }, 1000);
            } else {
                setError(`Unexpected payment status: ${result.paymentIntent.status}`);
            }
        } catch (err) {
            console.error('Stripe confirmation error:', err);
            setError(err.message || 'Payment failed');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="mt-6">
            <div className="text-center mb-6">
                <div className="w-12 h-12 bg-blue-600 rounded-xl flex items-center justify-center mx-auto mb-3">
                    <span className="text-white font-bold">💳</span>
                </div>
                <h3 className="text-xl font-bold text-gray-800 mb-2">Pay with Stripe</h3>
                <p className="text-gray-600">Secure payment processing • {currency}</p>
            </div>
            <form onSubmit={handleSubmit}>
                <div className="mb-6">
                    <CardElement
                        options={{
                            style: {
                                base: {
                                    fontSize: '18px',
                                    color: '#424770',
                                    fontFamily: 'system-ui, sans-serif',
                                    '::placeholder': { color: '#aab7c4' },
                                    padding: '12px',
                                },
                                invalid: { color: '#9e2146' },
                            },
                        }}
                        className="p-4 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                    />
                </div>
                <button
                    type="submit"
                    disabled={!stripe || loading}
                    className="w-full bg-gradient-to-r from-blue-600 to-indigo-600 text-white p-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 disabled:from-gray-400 disabled:to-gray-400 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                >
                    {loading ? 'Processing...' : `Pay with Stripe (${currency})`}
                </button>
                {error && (
                    <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                        <p className="text-red-600 text-center text-sm">{error}</p>
                    </div>
                )}
                {success && (
                    <div className="mt-4 p-3 bg-green-50 border border-green-200 rounded-lg">
                        <p className="text-green-600 text-center text-sm">Payment initiated, processing...</p>
                    </div>
                )}
            </form>
        </div>
    );
}

export default StripePayment;