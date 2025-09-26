//
//
import React, { useState } from 'react';
import { PayPalButtons } from '@paypal/react-paypal-js';
import axios from 'axios';

function PayPalPayment({ paymentId, transactionId, currency }) {
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(false);

    console.log('PayPal paymentId:', paymentId);
    console.log('PayPal transactionId:', transactionId);

    return (
        <div style={{ marginTop: '20px' }}>
            <h3 className="text-xl font-bold mb-4 text-center">Pay with PayPal ({currency})</h3>
            <PayPalButtons
                createOrder={() => paymentId}
                onApprove={async (data, actions) => {
                    console.log('PayPal onApprove data.orderID:', data.orderID);
                    setLoading(true);
                    setError(null);
                    try {
                        const response = await axios.post(
                            `${import.meta.env.VITE_API_URL}/api/paypal/capture`,
                            { order_id: data.orderID, transaction_id: transactionId },
                            {
                                headers: {
                                    'Content-Type': 'application/json',
                                    'Authorization': `Bearer ${localStorage.getItem('jwt_token')}`,
                                },
                            }
                        );
                        console.log('Capture response:', response.data);
                        if (response.data.status === 'completed') {
                            window.location.href = `/success?transaction_id=${transactionId}`;
                        } else {
                            setError(response.data.error_message || 'PayPal payment failed');
                        }
                    } catch (err) {
                        console.error('Capture error:', err);
                        setError(
                            err.response?.data?.message ||
                            err.response?.data?.error_message ||
                            err.message ||
                            'PayPal capture failed'
                        );
                    } finally {
                        setLoading(false);
                    }
                }}
                onError={(err) => {
                    console.error('PayPal SDK error:', err);
                    setError(err.message || 'PayPal payment failed');
                }}
                style={{ layout: 'vertical' }}
                disabled={loading}
            />
            {error && <p style={{ color: 'red' }}>{error}</p>}
            {loading && <p>Processing PayPal payment...</p>}
        </div>
    );
}

export default PayPalPayment;
//

//================================

// import React, { useState } from 'react';
// import { PayPalButtons } from '@paypal/react-paypal-js';
// import axios from 'axios';
//
// function PayPalPayment({ paymentId, transactionId, currency }) {
//     const [error, setError] = useState(null);
//     const [loading, setLoading] = useState(false);
//
//     console.log('PayPal paymentId:', paymentId);
//     console.log('PayPal transactionId:', transactionId);
//     console.log('PayPal currency:', currency);
//
//     return (
//         <div className="max-w-md mx-auto mt-6 p-6 bg-white rounded-lg shadow-md">
//             <h3 className="text-xl font-bold mb-4 text-center">Pay with PayPal ({currency})</h3>
//             <PayPalButtons
//                 // Remove createOrder since the order is already created
//                 onApprove={async (data, actions) => {
//                     console.log('PayPal onApprove data.orderID:', data.orderID);
//                     setLoading(true);
//                     setError(null);
//                     try {
//                         const response = await axios.post(
//                             `${import.meta.env.VITE_API_URL}/api/paypal/capture`,
//                             { order_id: data.orderID, transaction_id: transactionId },
//                             {
//                                 headers: {
//                                     'Content-Type': 'application/json',
//                                     'Authorization': `Bearer ${localStorage.getItem('jwt_token')}`,
//                                 },
//                             }
//                         );
//                         console.log('Capture response:', response.data);
//                         if (response.data.status === 'completed') {
//                             window.location.href = `/success?transaction_id=${transactionId}`;
//                         } else {
//                             setError(response.data.error_message || 'PayPal payment failed');
//                         }
//                     } catch (err) {
//                         console.error('Capture error:', err);
//                         setError(
//                             err.response?.data?.message ||
//                             err.response?.data?.error_message ||
//                             err.message ||
//                             'PayPal capture failed'
//                         );
//                     } finally {
//                         setLoading(false);
//                     }
//                 }}
//                 onError={(err) => {
//                     console.error('PayPal SDK error:', err);
//                     setError(err.message || 'PayPal payment failed');
//                 }}
//                 style={{ layout: 'vertical' }}
//                 disabled={loading}
//                 fundingSource="paypal"
//             />
//             {error && <p className="text-red-500 mt-4 text-center">{error}</p>}
//             {loading && <p className="text-center">Processing PayPal payment...</p>}
//         </div>
//     );
// }
//
// export default PayPalPayment;