// import React, { useState } from 'react';
// import { CardElement, useStripe, useElements } from '@stripe/react-stripe-js';
//
// function StripePayment({ clientSecret, transactionId }) {
//     const stripe = useStripe();
//     const elements = useElements();
//     const [error, setError] = useState(null);
//     const [processing, setProcessing] = useState(false);
//
//     console.log('StripePayment clientSecret:', clientSecret);
//     console.log('StripePayment transactionId:', transactionId);
//
//     const handleSubmit = async (event) => {
//         event.preventDefault();
//         setProcessing(true);
//         setError(null);
//
//         if (!stripe || !elements) {
//             setError('Stripe.js has not loaded');
//             setProcessing(false);
//             return;
//         }
//
//         const result = await stripe.confirmCardPayment(clientSecret, {
//             payment_method: {
//                 card: elements.getElement(CardElement),
//                 billing_details: {
//                     name: 'Test User', // Replace with user data if available
//                 },
//             },
//         });
//
//         console.log('confirmCardPayment result:', result);
//
//         if (result.error) {
//             setError(result.error.message);
//             setProcessing(false);
//         } else if (result.paymentIntent.status === 'succeeded') {
//             window.location.href = `/success?transaction_id=${transactionId}`;
//         } else {
//             setError(`Unexpected payment intent status: ${result.paymentIntent.status}`);
//             setProcessing(false);
//         }
//     };
//
//     return (
//         <form onSubmit={handleSubmit} style={{ marginTop: '20px' }}>
//             <CardElement
//                 options={{
//                     style: {
//                         base: {
//                             fontSize: '16px',
//                             color: '#424770',
//                             '::placeholder': { color: '#aab7c4' },
//                         },
//                         invalid: { color: '#9e2146' },
//                     },
//                 }}
//             />
//             <button
//                 type="submit"
//                 disabled={processing || !stripe || !elements}
//                 style={{ margin: '10px 0', padding: '10px', width: '100%' }}
//             >
//                 {processing ? 'Processing...' : 'Pay with Stripe'}
//             </button>
//             {error && <p style={{ color: 'red' }}>{error}</p>}
//         </form>
//     );
// }
//
// export default StripePayment;



import React, { useState } from 'react';
import { CardElement, useStripe, useElements } from '@stripe/react-stripe-js';

function StripePayment({ clientSecret, transactionId }) {
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
                        name: 'Test User',
                    },
                },
            });

            if (result.error) {
                setError(result.error.message);
            } else if (result.paymentIntent.status === 'succeeded') {
                setSuccess(true);
                // Redirect after a short delay to allow webhook to process
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
        <form onSubmit={handleSubmit} style={{ marginTop: '20px' }}>
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
            />
            <button
                type="submit"
                disabled={!stripe || loading}
                style={{ margin: '10px', padding: '10px', width: '100%' }}
            >
                {loading ? 'Processing...' : 'Pay with Stripe'}
            </button>
            {error && <p style={{ color: 'red' }}>{error}</p>}
            {success && <p style={{ color: 'green' }}>Payment initiated, processing...</p>}
        </form>
    );
}

export default StripePayment;