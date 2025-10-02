import React, { useState } from 'react';
import { PayPalButtons } from '@paypal/react-paypal-js';
import axios from 'axios';

function PayPalPayment({ paymentId, transactionId, currency, amount }) {
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(false);

    console.log('PayPal paymentId:', paymentId);
    console.log('PayPal transactionId:', transactionId);
    console.log('PayPal currency:', currency);
    console.log('PayPal amount:', amount);

    const getToken = () => localStorage.getItem('jwt_token') || sessionStorage.getItem('jwt_token');

    const getErrorMessage = (message) => {
        if (message.includes('INSTRUMENT_DECLINED')) {
            return 'Your payment method was declined. Try a different card or PayPal account!';
        }
        return message || 'PayPal payment crashed the Payego party!';
    };

    return (
        <div className="mt-6">
            <div className="text-center mb-6">
                <div className="w-12 h-12 bg-gradient-to-r from-purple-500 to-pink-500 rounded-xl flex items-center justify-center mx-auto mb-3">
                    <span className="text-white font-bold">PP</span>
                </div>
                <h3 className="text-xl font-bold text-gray-800 mb-2">Pay with PayPal</h3>
                <p className="text-gray-600 text-sm">Secure payment of {amount} {currency}</p>
            </div>
            {paymentId ? (
                <PayPalButtons
                    createOrder={() => {
                        console.log('Creating PayPal order with paymentId:', paymentId);
                        return Promise.resolve(paymentId);
                    }}
                    onApprove={async (data, actions) => {
                        console.log('PayPal onApprove data:', data);
                        setLoading(true);
                        setError(null);
                        try {
                            const token = getToken();
                            if (!token) {
                                throw new Error('No session found. Time to join the Payego party!');
                            }
                            console.log('Calling /api/paypal/capture with order_id:', data.orderID);
                            const response = await axios.post(
                                `${import.meta.env.VITE_API_URL}/api/paypal/capture`,
                                { order_id: data.orderID, transaction_id: transactionId },
                                {
                                    headers: {
                                        'Content-Type': 'application/json',
                                        'Authorization': `Bearer ${token}`,
                                    },
                                }
                            );
                            console.log('Capture response:', response.data);
                            if (response.data.status === 'completed') {
                                window.location.href = `/success?transaction_id=${transactionId}`;
                            } else {
                                setError(getErrorMessage(response.data.error_message));
                            }
                        } catch (err) {
                            console.error('Capture error:', err);
                            console.error('Capture error response:', err.response?.data);
                            console.error('Capture error status:', err.response?.status);
                            setError(getErrorMessage(err.response?.data?.error_message));
                        } finally {
                            setLoading(false);
                        }
                    }}
                    onError={(err) => {
                        console.error('PayPal SDK error:', err);
                        setError('PayPal payment tripped over its own feet! Try again.');
                    }}
                    style={{ layout: 'vertical', color: 'blue', label: 'pay' }}
                    disabled={loading}
                />
            ) : (
                <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                    <p className="text-red-600 text-center text-sm">
                        No PayPal payment ID provided. Did it wander off to Vegas?
                    </p>
                </div>
            )}
            {error && (
                <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                    <p className="text-red-600 text-center text-sm">{error}</p>
                </div>
            )}
            {loading && (
                <div className="mt-4 p-3 bg-purple-50 border border-purple-200 rounded-lg">
                    <p className="text-purple-600 text-center text-sm">Processing PayPal payment...</p>
                </div>
            )}
        </div>
    );
}

export default PayPalPayment;



//
// import React, { useState, useEffect } from 'react';
// import axios from 'axios';
// import { useLocation } from 'react-router-dom';
//
// function PayPalPayment({ transactionId }) {
//     const [error, setError] = useState(null);
//     const [loading, setLoading] = useState(false);
//     const location = useLocation();
//
//     const getToken = () => localStorage.getItem('jwt_token') || sessionStorage.getItem('jwt_token');
//
//     useEffect(() => {
//         const capturePayPalPayment = async () => {
//             const params = new URLSearchParams(location.search);
//             const orderId = params.get('order_id');
//             if (!orderId || !transactionId) {
//                 setError('Missing payment details. PayPal lost its party invite!');
//                 return;
//             }
//
//             setLoading(true);
//             setError(null);
//             try {
//                 const token = getToken();
//                 if (!token) {
//                     throw new Error('No session found. Time to join the Payego party!');
//                 }
//                 const response = await axios.post(
//                     `${import.meta.env.VITE_API_URL}/api/paypal/capture`,
//                     { order_id: orderId, transaction_id: transactionId },
//                     {
//                         headers: {
//                             'Content-Type': 'application/json',
//                             'Authorization': `Bearer ${token}`,
//                         },
//                     }
//                 );
//                 console.log('Capture response:', response.data);
//                 if (response.data.status !== 'completed') {
//                     setError(response.data.message || 'PayPal payment took a coffee break!');
//                 }
//             } catch (err) {
//                 console.error('Capture error:', err);
//                 setError(err.response?.data?.message || 'PayPal payment crashed the party!');
//             } finally {
//                 setLoading(false);
//             }
//         };
//
//         capturePayPalPayment();
//     }, [location, transactionId]);
//
//     return (
//         <div className="mt-6">
//             <div className="text-center mb-6">
//                 <div className="w-12 h-12 bg-yellow-500 rounded-xl flex items-center justify-center mx-auto mb-3">
//                     <span className="text-white font-bold">PP</span>
//                 </div>
//                 <h3 className="text-xl font-bold text-gray-800 mb-2">Processing PayPal Payment</h3>
//             </div>
//             {error && (
//                 <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
//                     <p className="text-red-600 text-center text-sm">{error}</p>
//                 </div>
//             )}
//             {loading && (
//                 <div className="mt-4 p-3 bg-blue-50 border border-blue-200 rounded-lg">
//                     <p className="text-blue-600 text-center text-sm">Processing PayPal payment...</p>
//                 </div>
//             )}
//         </div>
//     );
// }
//
// export default PayPalPayment;
