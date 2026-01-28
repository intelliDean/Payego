import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import { PayPalScriptProvider } from '@paypal/react-paypal-js';
import PayPalPayment from './PayPalPayment';
import ErrorBoundary from './ErrorBoundary';
import { transactionApi } from '../api/transactions';
import { Currency } from '../types';
import { getErrorMessage } from '../utils/errorHandler';

const topUpSchema = z.object({
    amount: z.number().min(1, 'Amount must be at least 1').max(10000, 'Amount must be at most 10,000'),
    provider: z.enum(['Stripe', 'Paypal']),
    currency: z.string().min(1, 'Please select a currency'),
});

type TopUpFormValues = z.infer<typeof topUpSchema>;

const SUPPORTED_CURRENCIES: Currency[] = ([
    'USD', 'EUR', 'GBP', 'NGN', 'CAD', 'AUD', 'CHF', 'JPY', 'CNY', 'SEK',
    'NZD', 'MXN', 'SGD', 'HKD', 'NOK', 'KRW', 'TRY', 'INR', 'BRL', 'ZAR'
].sort() as Currency[]);

const TopUpForm: React.FC = () => {
    const navigate = useNavigate();
    const [submitting, setSubmitting] = useState(false);
    const [paymentData, setPaymentData] = useState<any>(null);
    const [error, setError] = useState<string | null>(null);

    const {
        register,
        handleSubmit,
        watch,
        formState: { errors },
    } = useForm<TopUpFormValues>({
        resolver: zodResolver(topUpSchema),
        defaultValues: {
            amount: 10,
            provider: 'Stripe',
            currency: 'USD',
        }
    });

    const selectedProvider = watch('provider');
    const selectedCurrency = watch('currency');

    const onSubmit = async (data: TopUpFormValues) => {
        setSubmitting(true);
        setError(null);
        try {
            const response = await transactionApi.topUp(data);
            setPaymentData(response);
            if (data.provider === 'Stripe' && response.session_url) {
                window.location.assign(response.session_url);
            }
        } catch (err: any) {
            setError(getErrorMessage(err));
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <ErrorBoundary>
            <div className="max-w-md mx-auto mt-4 sm:mt-10 p-6 sm:p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
                <div className="text-center mb-8">
                    <div className="w-12 h-12 bg-gradient-to-r from-purple-500 to-pink-500 rounded-2xl flex items-center justify-center mx-auto mb-4">
                        <span className="text-white text-xl">💸</span>
                    </div>
                    <h2 className="text-2xl font-bold text-gray-800 mb-2">Top Up Account</h2>
                    <p className="text-gray-600 text-sm">Add funds to your wallet</p>
                </div>

                {!paymentData || selectedProvider === 'Stripe' ? (
                    <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
                        <div>
                            <label className="input-label">Amount</label>
                            <input
                                type="number"
                                step="0.01"
                                {...register('amount', { valueAsNumber: true })}
                                className="input-with-icon"
                                placeholder="10.00"
                            />
                            {errors.amount && <p className="mt-1 text-xs text-red-500">{errors.amount.message}</p>}
                        </div>

                        <div>
                            <label className="input-label">Provider</label>
                            <select {...register('provider')} className="input-with-icon">
                                <option value="Stripe">Stripe</option>
                                <option value="Paypal">PayPal</option>
                            </select>
                        </div>

                        <div>
                            <label className="input-label">Currency</label>
                            <select {...register('currency')} className="input-with-icon">
                                {SUPPORTED_CURRENCIES.map(c => (
                                    <option key={c} value={c}>{c}</option>
                                ))}
                            </select>
                        </div>

                        <div className="flex gap-4 pt-2">
                            <button
                                type="submit"
                                disabled={submitting}
                                className="flex-1 btn-primary-glow p-3 rounded-lg font-bold"
                            >
                                {submitting ? 'Processing...' : 'Proceed'}
                            </button>
                            <button
                                type="button"
                                onClick={() => navigate('/dashboard')}
                                className="flex-1 bg-gray-100 p-3 rounded-lg font-bold"
                            >
                                Cancel
                            </button>
                        </div>

                        {error && <div className="alert-error text-center text-sm">{error}</div>}
                    </form>
                ) : (
                    <PayPalScriptProvider
                        options={{
                            clientId: import.meta.env.VITE_PAYPAL_CLIENT_ID || 'test',
                            currency: selectedCurrency,
                        }}
                    >
                        <PayPalPayment
                            paymentId={paymentData.payment_id}
                            transactionId={paymentData.transaction_id}
                            currency={selectedCurrency}
                            amount={paymentData.amount}
                        />
                    </PayPalScriptProvider>
                )}
            </div>
        </ErrorBoundary>
    );
};

export default TopUpForm;
