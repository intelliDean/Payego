// import React, { useState, useEffect, useRef } from "react";
// import axios from "axios";
// import { PayPalScriptProvider } from "@paypal/react-paypal-js";
// import { useNavigate } from "react-router-dom";
// import PayPalPayment from "./PayPalPayment";
// import ErrorBoundary from "./ErrorBoundary";
//
// const SUPPORTED_CURRENCIES = [
//     "USD", "EUR", "GBP", "AUD", "BRL", "CAD", "CHF", "CNY", "HKD", "INR",
//     "JPY", "KRW", "MXN", "NGN", "NOK", "NZD", "SEK", "SGD", "TRY", "ZAR",
// ].sort((a, b) => a.localeCompare(b, "en", { sensitivity: "base" }));
//
// function TopUpForm() {
//     const [amount, setAmount] = useState("");
//     const [provider, setProvider] = useState("stripe");
//     const [currency, setCurrency] = useState("USD");
//     const [paymentData, setPaymentData] = useState(null);
//     const [error, setError] = useState(null);
//     const [loading, setLoading] = useState(true);
//     const [submitting, setSubmitting] = useState(false);
//     const navigate = useNavigate();
//     const amountInputRef = useRef(null);
//     const currencySelectRef = useRef(null);
//
//     useEffect(() => {
//         const validateToken = async () => {
//             try {
//                 const token = localStorage.getItem("jwt_token") || sessionStorage.getItem("jwt_token");
//                 if (!token) {
//                     setError("No session found. Time to join the Payego party!");
//                     navigate("/login");
//                     return;
//                 }
//
//                 await axios.get(`${import.meta.env.VITE_API_URL}/api/current_user`, {
//                     headers: { Authorization: `Bearer ${token}` },
//                 });
//
//                 console.log("TopUpForm initial state:", { amount, provider, currency });
//             } catch (err) {
//                 if (err.response?.status === 401) {
//                     setError("Session expired. Back to the login gate!");
//                     localStorage.removeItem("jwt_token");
//                     sessionStorage.removeItem("jwt_token");
//                     navigate("/login");
//                 } else {
//                     setError(err.response?.data?.message || "Setup ran off to Vegas!");
//                 }
//             } finally {
//                 setLoading(false);
//             }
//         };
//         validateToken();
//     }, [navigate]);
//
//     useEffect(() => {
//         if (paymentData?.session_url) {
//             console.log("Attempting redirect to Stripe Checkout:", paymentData.session_url);
//             try {
//                 window.location.assign(paymentData.session_url);
//             } catch (err) {
//                 console.error("Redirect failed:", err);
//                 setError("Failed to redirect to Stripe. The payment page ditched us!");
//             }
//         }
//     }, [paymentData]);
//
//     const validateAmount = (value) => {
//         const num = parseFloat(value);
//         if (isNaN(num) || num < 1 || num > 10000) {
//             return "Amount must be between 1 and 10,000";
//         }
//         return null;
//     };
//
//     const validateCurrency = (value) => {
//         if (!SUPPORTED_CURRENCIES.includes(value)) {
//             return "Invalid currency selected";
//         }
//         return null;
//     };
//
//     const handleAmountChange = (e) => {
//         const value = e.target.value;
//         setAmount(value);
//         setError(validateAmount(value));
//     };
//
//     const handleCurrencyChange = (e) => {
//         const value = e.target.value;
//         setCurrency(value);
//         setError(validateCurrency(value));
//     };
//
//     const handleProviderChange = (e) => {
//         setProvider(e.target.value);
//         setPaymentData(null);
//     };
//
//     const handleSubmit = async (e) => {
//         e.preventDefault();
//         setSubmitting(true);
//         setError(null);
//
//         const amountError = validateAmount(amount);
//         const currencyError = validateCurrency(currency);
//         if (amountError || currencyError) {
//             setError(amountError || currencyError);
//             setSubmitting(false);
//             if (amountError) amountInputRef.current?.focus();
//             else if (currencyError) currencySelectRef.current?.focus();
//             return;
//         }
//
//         try {
//             const token = localStorage.getItem("jwt_token") || sessionStorage.getItem("jwt_token");
//             const response = await axios.post(
//                 `${import.meta.env.VITE_API_URL}/api/top_up`,
//                 { amount: parseFloat(amount), provider, currency },
//                 {
//                     headers: {
//                         "Content-Type": "application/json",
//                         Authorization: `Bearer ${token}`,
//                     },
//                 }
//             );
//             console.log("TopUp API response:", response.data);
//             setPaymentData(response.data);
//             setSubmitting(false);
//         } catch (err) {
//             console.error("TopUp API error:", err);
//             setError(err.response?.data?.message || "Payment initiation crashed the party!");
//             setSubmitting(false);
//         }
//     };
//
//     const handleCancel = () => {
//         setPaymentData(null);
//         navigate("/");
//     };
//
//     return (
//         <ErrorBoundary>
//             <div className="min-h-screen bg-gray-50 flex items-center justify-center p-4">
//                 <div className="max-w-md mx-auto p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
//                     <div className="text-center mb-8">
//                         <div className="w-16 h-16 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-2xl flex items-center justify-center mx-auto mb-4">
//                             <svg className="w-8 h-8 text-white" viewBox="0 0 24 24" fill="currentColor" aria-hidden="true">
//                                 <path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm3.88-11.71L10 14.17l-1.88-1.88a1 1 0 0 0-1.41 1.41l2.59 2.59a1 1 0 0 0 1.41 0L17 10.41a1 1 0 0 0-1.12-1.41z" />
//                             </svg>
//                         </div>
//                         <h2 className="text-3xl font-bold text-gray-900 mb-2" id="top-up-form-title">Top Up Account</h2>
//                         <p className="text-gray-600 text-sm">Add funds to your wallet</p>
//                     </div>
//
//                     {loading ? (
//                         <div className="flex flex-col items-center justify-center py-8">
//                             <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
//                             <p className="mt-2 text-gray-600 text-sm">Setting up top-up form...</p>
//                         </div>
//                     ) : error ? (
//                         <div className="p-3 bg-red-50 border border-red-200 rounded-lg mb-4">
//                             <p className="text-red-600 text-center text-sm" id="error-message" role="alert">{error}</p>
//                         </div>
//                     ) : !paymentData ? (
//                         <form onSubmit={handleSubmit} aria-labelledby="top-up-form-title">
//                             <div className="mb-4">
//                                 <label htmlFor="amount" className="block text-gray-700 font-medium mb-2">Amount</label>
//                                 <input
//                                     id="amount"
//                                     type="number"
//                                     value={amount}
//                                     onChange={handleAmountChange}
//                                     min="1"
//                                     max="10000"
//                                     step="0.01"
//                                     className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-600 focus:border-transparent transition-all duration-200"
//                                     placeholder="Enter amount (1 - 10,000)"
//                                     required
//                                     aria-describedby={error ? "error-message" : undefined}
//                                     ref={amountInputRef}
//                                 />
//                             </div>
//                             <div className="mb-4">
//                                 <label htmlFor="provider" className="block text-gray-700 font-medium mb-2">Payment Provider</label>
//                                 <select
//                                     id="provider"
//                                     value={provider}
//                                     onChange={handleProviderChange}
//                                     className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-600 focus:border-transparent transition-all duration-200"
//                                     aria-describedby={error ? "error-message" : undefined}
//                                 >
//                                     <option value="stripe">Stripe</option>
//                                     <option value="paypal">PayPal</option>
//                                 </select>
//                             </div>
//                             <div className="mb-4">
//                                 <label htmlFor="currency" className="block text-gray-700 font-medium mb-2">Currency</label>
//                                 <select
//                                     id="currency"
//                                     value={currency}
//                                     onChange={handleCurrencyChange}
//                                     className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-600 focus:border-transparent transition-all duration-200"
//                                     aria-describedby={error ? "error-message" : undefined}
//                                     ref={currencySelectRef}
//                                 >
//                                     {SUPPORTED_CURRENCIES.map((curr) => (
//                                         <option key={curr} value={curr}>{curr}</option>
//                                     ))}
//                                 </select>
//                             </div>
//                             <div className="flex space-x-4">
//                                 <button
//                                     type="submit"
//                                     disabled={submitting || !!error}
//                                     className="flex-1 bg-gradient-to-r from-blue-600 to-indigo-600 text-white p-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 disabled:from-gray-400 disabled:to-gray-400 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
//                                     aria-label="Proceed to payment"
//                                 >
//                                     {submitting ? "Processing..." : "Proceed to Payment"}
//                                 </button>
//                                 <button
//                                     type="button"
//                                     onClick={handleCancel}
//                                     disabled={submitting}
//                                     className="flex-1 bg-gray-200 text-gray-700 p-3 rounded-lg hover:bg-gray-300 disabled:bg-gray-300 disabled:text-gray-500 transition-all duration-200 font-medium"
//                                     aria-label="Cancel and return to dashboard"
//                                 >
//                                     Cancel
//                                 </button>
//                             </div>
//                             {error && (
//                                 <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
//                                     <p className="text-red-600 text-center text-sm" id="error-message" role="alert">{error}</p>
//                                 </div>
//                             )}
//                         </form>
//                     ) : provider === "stripe" && paymentData.session_url ? (
//                         <div className="flex flex-col items-center justify-center py-8">
//                             <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
//                             <p className="mt-2 text-gray-600 text-sm">Redirecting to Stripe Checkout...</p>
//                         </div>
//                     ) : null}
//                     {paymentData && provider === "paypal" && (
//                         <PayPalScriptProvider
//                             options={{
//                                 clientId: import.meta.env.VITE_PAYPAL_CLIENT_ID,
//                                 currency: currency,
//                             }}
//                         >
//                             <PayPalPayment
//                                 paymentId={paymentData.payment_id}
//                                 transactionId={paymentData.transaction_id}
//                                 currency={currency}
//                                 amount={paymentData.amount}
//                             />
//                         </PayPalScriptProvider>
//                     )}
//                 </div>
//             </div>
//         </ErrorBoundary>
//     );
// }
//
// export default TopUpForm;




import React, { useState, useEffect, useRef } from 'react';
import axios from 'axios';
import { PayPalScriptProvider } from '@paypal/react-paypal-js';
import { useNavigate } from 'react-router-dom';
import PayPalPayment from './PayPalPayment';
import ErrorBoundary from './ErrorBoundary';

const SUPPORTED_CURRENCIES = [
    'USD', 'EUR', 'GBP', 'AUD', 'BRL', 'CAD', 'CHF', 'CNY', 'HKD', 'INR',
    'JPY', 'KRW', 'MXN', 'NGN', 'NOK', 'NZD', 'SEK', 'SGD', 'TRY', 'ZAR',
].sort((a, b) => a.localeCompare(b, 'en', { sensitivity: 'base' }));

const SUPPORTED_PROVIDERS = ['stripe', 'paypal'];

function TopUpForm() {
    const [amount, setAmount] = useState('');
    const [provider, setProvider] = useState('stripe');
    const [currency, setCurrency] = useState('USD');
    const [paymentData, setPaymentData] = useState(null);
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(true);
    const [submitting, setSubmitting] = useState(false);
    const navigate = useNavigate();
    const amountInputRef = useRef(null);
    const currencySelectRef = useRef(null);

    useEffect(() => {
        const validateToken = async () => {
            try {
                const token = localStorage.getItem('jwt_token') || sessionStorage.getItem('jwt_token');
                if (!token) {
                    setError('No session found. Time to join the Payego party!');
                    navigate('/login');
                    return;
                }

                await axios.get(`${import.meta.env.VITE_API_URL}/api/current_user`, {
                    headers: { Authorization: `Bearer ${token}` },
                });

                console.log('TopUpForm initial state:', { amount, provider, currency });
            } catch (err) {
                if (err.response?.status === 401) {
                    setError('Session expired. Back to the login gate!');
                    localStorage.removeItem('jwt_token');
                    sessionStorage.removeItem('jwt_token');
                    navigate('/login');
                } else {
                    setError(err.response?.data?.message || 'Setup ran off to Vegas!');
                }
            } finally {
                setLoading(false);
            }
        };
        validateToken();
    }, [navigate]);

    useEffect(() => {
        if (paymentData?.session_url) {
            console.log('Attempting redirect to Stripe Checkout:', paymentData.session_url);
            try {
                window.location.assign(paymentData.session_url);
            } catch (err) {
                console.error('Redirect failed:', err);
                setError('Failed to redirect to Stripe. The payment page ditched us!');
            }
        }
    }, [paymentData]);

    const validateAmount = (value) => {
        const num = parseFloat(value);
        if (isNaN(num) || num < 1 || num > 10000) {
            return 'Amount must be between 1 and 10,000';
        }
        return null;
    };

    const validateCurrency = (value) => {
        if (!SUPPORTED_CURRENCIES.includes(value)) {
            return 'Invalid currency selected';
        }
        return null;
    };

    const handleAmountChange = (e) => {
        const value = e.target.value;
        setAmount(value);
        setError(validateAmount(value));
    };

    const handleCurrencyChange = (e) => {
        const value = e.target.value;
        setCurrency(value);
        setError(validateCurrency(value));
    };

    const handleProviderChange = (e) => {
        setProvider(e.target.value);
        setPaymentData(null);
    };

    const handleSubmit = async (e) => {
        e.preventDefault();
        setSubmitting(true);
        setError(null);

        const amountError = validateAmount(amount);
        const currencyError = validateCurrency(currency);
        if (amountError || currencyError) {
            setError(amountError || currencyError);
            setSubmitting(false);
            if (amountError) amountInputRef.current?.focus();
            else if (currencyError) currencySelectRef.current?.focus();
            return;
        }

        try {
            const token = localStorage.getItem('jwt_token') || sessionStorage.getItem('jwt_token');
            const response = await axios.post(
                `${import.meta.env.VITE_API_URL}/api/top_up`,
                { amount: parseFloat(amount), provider, currency },
                {
                    headers: {
                        'Content-Type': 'application/json',
                        Authorization: `Bearer ${token}`,
                    },
                }
            );
            console.log('TopUp API response:', response.data);
            setPaymentData(response.data);
            setSubmitting(false);
        } catch (err) {
            console.error('TopUp API error:', err);
            setError(err.response?.data?.message || 'Payment initiation crashed the Payego party!');
            setSubmitting(false);
        }
    };

    const handleCancel = () => {
        setPaymentData(null);
        navigate('/dashboard');
    };

    return (
        <ErrorBoundary>
            <div className="max-w-md mx-auto mt-4 sm:mt-10 p-6 sm:p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
                <div className="text-center mb-8">
                    <div className="w-12 sm:w-16 h-12 sm:h-16 bg-gradient-to-r from-purple-500 to-pink-500 rounded-2xl flex items-center justify-center mx-auto mb-4">
                        <span className="text-white text-xl sm:text-2xl">💸</span>
                    </div>
                    <h2 className="text-2xl sm:text-3xl font-bold text-gray-800 mb-2" id="top-up-form-title">Top Up Account</h2>
                    <p className="text-gray-600 text-sm sm:text-base">Add funds to your wallet</p>
                </div>

                {loading ? (
                    <div className="flex flex-col items-center justify-center py-8">
                        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-500"></div>
                        <p className="mt-2 text-gray-600 text-sm">Setting up top-up form...</p>
                    </div>
                ) : error ? (
                    <div className="p-3 bg-red-50 border border-red-200 rounded-lg mb-4">
                        <p className="text-red-600 text-center text-sm" id="error-message" role="alert">{error}</p>
                    </div>
                ) : !paymentData ? (
                    <form onSubmit={handleSubmit} aria-labelledby="top-up-form-title">
                        <div className="mb-4">
                            <label htmlFor="amount" className="block text-gray-700 font-medium mb-2">Amount</label>
                            <input
                                id="amount"
                                type="number"
                                value={amount}
                                onChange={handleAmountChange}
                                min="1"
                                max="10000"
                                step="0.01"
                                className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent transition-all duration-200"
                                placeholder="Enter amount (1 - 10,000)"
                                required
                                aria-describedby={error ? "error-message" : undefined}
                                ref={amountInputRef}
                            />
                        </div>
                        <div className="mb-4">
                            <label htmlFor="provider" className="block text-gray-700 font-medium mb-2">Payment Provider</label>
                            <select
                                id="provider"
                                value={provider}
                                onChange={handleProviderChange}
                                className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent transition-all duration-200"
                                aria-describedby={error ? "error-message" : undefined}
                            >
                                <option value="stripe">Stripe</option>
                                <option value="paypal">PayPal</option>
                            </select>
                        </div>
                        <div className="mb-4">
                            <label htmlFor="currency" className="block text-gray-700 font-medium mb-2">Currency</label>
                            <select
                                id="currency"
                                value={currency}
                                onChange={handleCurrencyChange}
                                className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500 focus:border-transparent transition-all duration-200"
                                aria-describedby={error ? "error-message" : undefined}
                                ref={currencySelectRef}
                            >
                                {SUPPORTED_CURRENCIES.map((curr) => (
                                    <option key={curr} value={curr}>{curr}</option>
                                ))}
                            </select>
                        </div>
                        <div className="flex space-x-4">
                            <button
                                type="submit"
                                disabled={submitting || !!error}
                                className="flex-1 bg-gradient-to-r from-purple-500 to-pink-500 text-white p-3 rounded-lg hover:from-purple-600 hover:to-pink-600 disabled:from-gray-400 disabled:to-gray-400 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                                aria-label="Proceed to payment"
                            >
                                {submitting ? 'Processing...' : 'Proceed to Payment'}
                            </button>
                            <button
                                type="button"
                                onClick={handleCancel}
                                disabled={submitting}
                                className="flex-1 bg-gray-200 text-gray-700 p-3 rounded-lg hover:bg-gray-300 disabled:bg-gray-300 disabled:text-gray-500 transition-all duration-200 font-medium"
                                aria-label="Cancel and return to dashboard"
                            >
                                Cancel
                            </button>
                        </div>
                    </form>
                ) : provider === 'stripe' && paymentData.session_url ? (
                    <div className="flex flex-col items-center justify-center py-8">
                        <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-purple-500"></div>
                        <p className="mt-2 text-gray-600 text-sm">Redirecting to Stripe Checkout...</p>
                    </div>
                ) : null}
                {paymentData && provider === 'paypal' && (
                    <PayPalScriptProvider
                        options={{
                            clientId: import.meta.env.VITE_PAYPAL_CLIENT_ID || 'test',
                            currency: currency,
                            intent: 'capture',
                        }}
                    >
                        <PayPalPayment
                            paymentId={paymentData.payment_id}
                            transactionId={paymentData.transaction_id}
                            currency={currency}
                            amount={paymentData.amount}
                        />
                    </PayPalScriptProvider>
                )}
            </div>
        </ErrorBoundary>
    );
}

export default TopUpForm;
