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
//
//     return (
//         <div className="mt-6">
//             <div className="text-center mb-6">
//                 <div className="w-12 h-12 bg-yellow-500 rounded-xl flex items-center justify-center mx-auto mb-3">
//                     <span className="text-white font-bold">PP</span>
//                 </div>
//                 <h3 className="text-xl font-bold text-gray-800 mb-2">Pay with PayPal</h3>
//                 <p className="text-gray-600">Secure payment processing • {currency}</p>
//             </div>
//             <PayPalButtons
//                 createOrder={() => paymentId}
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
//             />
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



/*
IMPORTANT
 import { PayPalScriptProvider } from "@paypal/react-paypal-js";

function App() {
    return (
        <PayPalScriptProvider options={{ "client-id": import.meta.env.VITE_PAYPAL_CLIENT_ID }}>
            <Routes>
                <Route path="/top-up" element={isAuthenticated ? <PayPalPayment {...props} /> : <Navigate to="/login" />} />
                <Route path="/success" element={isAuthenticated ? <Success /> : <Navigate to="/login" />} />

</Routes>
</PayPalScriptProvider>
);
}
/*/




import React, { useState } from 'react';
import { PayPalButtons } from '@paypal/react-paypal-js';
import axios from 'axios';

function PayPalPayment({ paymentId, transactionId, currency }) {
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(false);

    console.log('PayPal paymentId:', paymentId);
    console.log('PayPal transactionId:', transactionId);

    const getToken = () => localStorage.getItem('jwt_token') || sessionStorage.getItem('jwt_token');

    return (
        <div className="mt-6">
            <div className="text-center mb-6">
                <div className="w-12 h-12 bg-yellow-500 rounded-xl flex items-center justify-center mx-auto mb-3">
                    <span className="text-white font-bold">PP</span>
                </div>
                <h3 className="text-xl font-bold text-gray-800 mb-2">Pay with PayPal</h3>
                <p className="text-gray-600">Secure payment processing • {currency}</p>
            </div>
            <PayPalButtons
                createOrder={() => {
                    // Since /api/top_up already created the order, return the paymentId
                    return Promise.resolve(paymentId);
                }}
                onApprove={async (data, actions) => {
                    console.log('PayPal onApprove data.orderID:', data.orderID);
                    setLoading(true);
                    setError(null);
                    try {
                        const token = getToken();
                        if (!token) {
                            throw new Error('No session found. Time to join the Payego party!');
                        }
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
                            setError(response.data.message || 'PayPal payment took a coffee break!');
                        }
                    } catch (err) {
                        console.error('Capture error:', err);
                        setError(err.response?.data?.message || 'PayPal payment crashed the party!');
                    } finally {
                        setLoading(false);
                    }
                }}
                onError={(err) => {
                    console.error('PayPal SDK error:', err);
                    setError('PayPal payment tripped over its own feet!');
                }}
                style={{ layout: 'vertical' }}
                disabled={loading}
            />
            {error && (
                <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                    <p className="text-red-600 text-center text-sm">{error}</p>
                </div>
            )}
            {loading && (
                <div className="mt-4 p-3 bg-blue-50 border border-blue-200 rounded-lg">
                    <p className="text-blue-600 text-center text-sm">Processing PayPal payment...</p>
                </div>
            )}
        </div>
    );
}

export default PayPalPayment;
