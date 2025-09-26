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
        <div className="max-w-md mx-auto mt-6 p-6 bg-white rounded-lg shadow-md">
            <h3 className="text-xl font-bold mb-4 text-center">Pay with Stripe ({currency})</h3>
            <form onSubmit={handleSubmit}>
                <div className="mb-4">
                    <CardElement
                        options={{
                            style: {
                                base: {
                                    fontSize: '16px',
                                    color: '#424770',
                                    '::placeholder': { color: '#aab7c4' },
                                },
                                invalid: { color: '#9e2146' },
                            },
                        }}
                        className="p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                    />
                </div>
                <button
                    type="submit"
                    disabled={!stripe || loading}
                    className="w-full bg-blue-600 text-white p-2 rounded hover:bg-blue-700 disabled:bg-blue-300"
                >
                    {loading ? 'Processing...' : `Pay with Stripe (${currency})`}
                </button>
                {error && <p className="text-red-500 mt-4 text-center">{error}</p>}
                {success && <p className="text-green-500 mt-4 text-center">Payment initiated, processing...</p>}
            </form>
        </div>
    );
}

export default StripePayment;