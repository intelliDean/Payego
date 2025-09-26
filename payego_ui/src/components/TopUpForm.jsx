import React, { useState } from 'react';
import axios from 'axios';
import { loadStripe } from '@stripe/stripe-js';
import { Elements } from '@stripe/react-stripe-js';
import { PayPalScriptProvider } from '@paypal/react-paypal-js';
import StripePayment from './StripePayment';
import PayPalPayment from './PayPalPayment';

const stripePromise = loadStripe(import.meta.env.VITE_STRIPE_PUBLISHABLE_KEY);

const SUPPORTED_CURRENCIES = [
    'USD', 'NGN', 'GBP', 'EUR', 'CAD', 'AUD', 'JPY', 'CHF', 'CNY',
    'SEK', 'NZD', 'MXN', 'SGD', 'HKD', 'NOK', 'KRW', 'TRY', 'INR', 'BRL', 'ZAR'
];

function TopUpForm() {
    const [amount, setAmount] = useState('');
    const [provider, setProvider] = useState('stripe');
    const [currency, setCurrency] = useState('USD');
    const [paymentData, setPaymentData] = useState(null);
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(false);

    const handleSubmit = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);

        if (!amount || amount < 1 || amount > 10000) {
            setError('Amount must be between 1 and 10,000');
            setLoading(false);
            return;
        }

        if (!SUPPORTED_CURRENCIES.includes(currency)) {
            setError('Invalid currency selected');
            setLoading(false);
            return;
        }

        try {
            const response = await axios.post(
                `${import.meta.env.VITE_API_URL}/api/top_up`,
                { amount: parseFloat(amount), provider, currency },
                {
                    headers: {
                        'Content-Type': 'application/json',
                        'Authorization': `Bearer ${localStorage.getItem('jwt_token')}`,
                    },
                }
            );
            console.log('TopUp response:', response.data);
            setPaymentData(response.data);
        } catch (err) {
            console.error('TopUp error:', err);
            setError(err.response?.data?.message || 'Failed to initiate payment');
            setLoading(false);
        }
    };

    return (
        <div className="max-w-md mx-auto mt-10 p-6 bg-white rounded-lg shadow-md">
            <h2 className="text-2xl font-bold mb-6 text-center">Top Up Account</h2>
            {!paymentData ? (
                <form onSubmit={handleSubmit}>
                    <div className="mb-4">
                        <label className="block text-gray-700">Amount</label>
                        <input
                            type="number"
                            value={amount}
                            onChange={(e) => setAmount(e.target.value)}
                            min="1"
                            max="10000"
                            step="0.01"
                            className="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                            required
                        />
                    </div>
                    <div className="mb-4">
                        <label className="block text-gray-700">Payment Provider</label>
                        <select
                            value={provider}
                            onChange={(e) => setProvider(e.target.value)}
                            className="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                        >
                            <option value="stripe">Stripe</option>
                            <option value="paypal">PayPal</option>
                        </select>
                    </div>
                    <div className="mb-4">
                        <label className="block text-gray-700">Currency</label>
                        <select
                            value={currency}
                            onChange={(e) => setCurrency(e.target.value)}
                            className="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                        >
                            {SUPPORTED_CURRENCIES.map((curr) => (
                                <option key={curr} value={curr}>{curr}</option>
                            ))}
                        </select>
                    </div>
                    <button
                        type="submit"
                        disabled={loading}
                        className="w-full bg-blue-600 text-white p-2 rounded hover:bg-blue-700 disabled:bg-blue-300"
                    >
                        {loading ? 'Processing...' : 'Proceed to Payment'}
                    </button>
                    {error && <p className="text-red-500 mt-4 text-center">{error}</p>}
                </form>
            ) : provider === 'stripe' ? (
                <Elements stripe={stripePromise}>
                    <StripePayment
                        clientSecret={paymentData.payment_id}
                        transactionId={paymentData.transaction_id}
                        currency={currency}
                    />
                </Elements>
            ) : (
                <PayPalScriptProvider
                    options={{
                        clientId: import.meta.env.VITE_PAYPAL_CLIENT_ID,
                        currency: currency,
                    }}
                >
                    <PayPalPayment
                        paymentId={paymentData.payment_id}
                        transactionId={paymentData.transaction_id}
                        currency={currency}
                    />
                </PayPalScriptProvider>
            )}
        </div>
    );
}

export default TopUpForm;