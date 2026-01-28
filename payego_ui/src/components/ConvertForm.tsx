import React, { useState } from "react";
import { useQueryClient } from '@tanstack/react-query';
import { useNavigate } from "react-router-dom";
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import ErrorBoundary from "./ErrorBoundary";
import { useWallets } from "../hooks/useWallets";
import { transactionApi } from "../api/transactions";
import { Currency } from "@/types";
import client from "../api/client";
import { getErrorMessage } from '../utils/errorHandler';

const convertSchema = z.object({
    amount: z.number().min(0.01).max(10000),
    fromCurrency: z.string().min(1),
    toCurrency: z.string().min(1),
}).refine(data => data.fromCurrency !== data.toCurrency, {
    message: "Cannot convert to same currency",
    path: ["toCurrency"]
});

type ConvertFormValues = z.infer<typeof convertSchema>;

const SUPPORTED_CURRENCIES: Currency[] = ([
    'USD', 'EUR', 'GBP', 'NGN', 'CAD', 'AUD', 'CHF', 'JPY', 'CNY', 'SEK',
    'NZD', 'MXN', 'SGD', 'HKD', 'NOK', 'KRW', 'TRY', 'INR', 'BRL', 'ZAR'
].sort() as Currency[]);

const ConvertForm: React.FC = () => {
    const navigate = useNavigate();
    const queryClient = useQueryClient();
    const { data: wallets, isLoading: fetching } = useWallets();
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [previewData, setPreviewData] = useState<{
        rate: number;
        convertedAmount: number;
        fee: number;
    } | null>(null);
    const [showConfirmation, setShowConfirmation] = useState(false);
    const [showSuccess, setShowSuccess] = useState(false);
    const [conversionResult, setConversionResult] = useState<any>(null);

    const {
        register,
        handleSubmit,
        watch,
        formState: { errors },
    } = useForm<ConvertFormValues>({
        resolver: zodResolver(convertSchema),
    });

    const onPreview = async (data: ConvertFormValues) => {
        setSubmitting(true);
        setError(null);
        try {
            // Fetch exchange rate
            const rateResponse = await client.get(`/api/exchange-rate?from=${data.fromCurrency}&to=${data.toCurrency}`);
            const rate = rateResponse.data.rate;

            // Calculate preview
            const convertedAmount = data.amount * rate;
            const fee = convertedAmount * 0.01; // 1% fee
            const netAmount = convertedAmount - fee;

            setPreviewData({
                rate,
                convertedAmount: netAmount,
                fee
            });
            setShowConfirmation(true);
        } catch (err: any) {
            setError(getErrorMessage(err));
        } finally {
            setSubmitting(false);
        }
    };

    const onConfirm = async () => {
        const data = watch();
        setSubmitting(true);
        setError(null);
        try {
            const result = await transactionApi.convertCurrency({
                amount: data.amount,
                from_currency: data.fromCurrency,
                to_currency: data.toCurrency,
                idempotency_key: crypto.randomUUID()
            });
            setConversionResult(result);
            setShowConfirmation(false);

            // Invalidate queries to refresh balance and history
            queryClient.invalidateQueries({ queryKey: ['wallets'] });
            queryClient.invalidateQueries({ queryKey: ['transactions'] });

            setShowSuccess(true);
        } catch (err: any) {
            setError(getErrorMessage(err));
            setShowConfirmation(false);
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <ErrorBoundary>
            <div className="max-w-md mx-auto mt-4 sm:mt-10 p-6 sm:p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
                <div className="text-center mb-8">
                    <div className="w-12 h-12 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-2xl flex items-center justify-center mx-auto mb-4">
                        <span className="text-white text-xl">🔄</span>
                    </div>
                    <h2 className="text-2xl font-bold text-gray-800">Convert Currency</h2>
                </div>

                {fetching ? (
                    <div className="flex justify-center py-12"><div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div></div>
                ) : (
                    <form onSubmit={handleSubmit(onPreview)} className="space-y-4">
                        <div>
                            <label className="input-label">From</label>
                            <select {...register('fromCurrency')} className="input-with-icon">
                                <option value="">Select Currency</option>
                                {wallets?.map(w => <option key={w.currency} value={w.currency}>{w.currency} (Bal: {w.balance / 100})</option>)}
                            </select>
                        </div>

                        <div>
                            <label className="input-label">To</label>
                            <select {...register('toCurrency')} className="input-with-icon">
                                <option value="">Select Currency</option>
                                {SUPPORTED_CURRENCIES.map(c => <option key={c} value={c}>{c}</option>)}
                            </select>
                            {errors.toCurrency && <p className="text-red-500 text-xs mt-1">{errors.toCurrency.message}</p>}
                        </div>

                        <div>
                            <label className="input-label">Amount</label>
                            <input type="number" step="0.01" {...register('amount', { valueAsNumber: true })} className="input-with-icon" />
                            {errors.amount && <p className="text-red-500 text-xs mt-1">{errors.amount.message}</p>}
                        </div>

                        <div className="flex gap-4 pt-2">
                            <button type="submit" disabled={submitting} className="flex-1 btn-primary p-3 rounded-lg font-bold">
                                {submitting ? 'Loading...' : 'Preview Conversion'}
                            </button>
                            <button type="button" onClick={() => navigate('/dashboard')} className="flex-1 bg-gray-100 p-3 rounded-lg font-bold">Cancel</button>
                        </div>

                        {error && <div className="alert-error text-center text-sm">{error}</div>}
                    </form>
                )}

                {/* Confirmation Modal */}
                {showConfirmation && previewData && (
                    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
                        <div className="bg-white rounded-xl p-6 max-w-sm w-full shadow-2xl">
                            <h3 className="text-xl font-bold mb-4">Confirm Conversion</h3>
                            <div className="space-y-3 mb-6">
                                <div className="flex justify-between">
                                    <span className="text-gray-600">From:</span>
                                    <span className="font-semibold">{watch('fromCurrency')} {watch('amount')}</span>
                                </div>
                                <div className="flex justify-between">
                                    <span className="text-gray-600">Exchange Rate:</span>
                                    <span className="font-semibold">1 {watch('fromCurrency')} = {previewData.rate.toFixed(4)} {watch('toCurrency')}</span>
                                </div>
                                <div className="flex justify-between">
                                    <span className="text-gray-600">Fee (1%):</span>
                                    <span className="font-semibold">{watch('toCurrency')} {previewData.fee.toFixed(2)}</span>
                                </div>
                                <div className="border-t pt-3 flex justify-between">
                                    <span className="text-gray-900 font-bold">You'll Receive:</span>
                                    <span className="text-green-600 font-bold">{watch('toCurrency')} {previewData.convertedAmount.toFixed(2)}</span>
                                </div>
                            </div>
                            <div className="flex gap-3">
                                <button
                                    onClick={() => setShowConfirmation(false)}
                                    className="flex-1 px-4 py-2 border border-gray-300 rounded-lg hover:bg-gray-50"
                                >
                                    Cancel
                                </button>
                                <button
                                    onClick={onConfirm}
                                    disabled={submitting}
                                    className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
                                >
                                    {submitting ? 'Converting...' : 'Confirm'}
                                </button>
                            </div>
                        </div>
                    </div>
                )}

                {/* Success Modal */}
                {showSuccess && conversionResult && (
                    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
                        <div className="bg-white rounded-xl p-6 max-w-sm w-full shadow-2xl text-center">
                            <div className="w-16 h-16 bg-gradient-to-r from-green-500 to-emerald-500 rounded-full flex items-center justify-center mx-auto mb-4">
                                <svg className="w-8 h-8 text-white" viewBox="0 0 24 24" fill="currentColor">
                                    <path d="M20.285 2l-11.285 11.567-5.286-5.011-3.714 3.716 9 8.728 15-15.285z" />
                                </svg>
                            </div>
                            <h3 className="text-2xl font-bold text-gray-900 mb-2">Conversion Successful!</h3>
                            <p className="text-gray-600 mb-6">Your currency has been converted</p>
                            <div className="bg-gray-50 rounded-lg p-4 mb-6 space-y-2 text-left">
                                <div className="flex justify-between text-sm">
                                    <span className="text-gray-600">Converted Amount:</span>
                                    <span className="font-semibold">{watch('toCurrency')} {conversionResult.converted_amount?.toFixed(2)}</span>
                                </div>
                                <div className="flex justify-between text-sm">
                                    <span className="text-gray-600">Exchange Rate:</span>
                                    <span className="font-semibold">{conversionResult.exchange_rate?.toFixed(4)}</span>
                                </div>
                                <div className="flex justify-between text-sm">
                                    <span className="text-gray-600">Fee:</span>
                                    <span className="font-semibold">{watch('toCurrency')} {conversionResult.fee?.toFixed(2)}</span>
                                </div>
                            </div>
                            <button
                                onClick={() => navigate('/dashboard')}
                                className="w-full bg-gradient-to-r from-blue-600 to-indigo-600 text-white px-6 py-3 rounded-lg font-medium shadow-lg hover:shadow-xl"
                            >
                                Return to Dashboard
                            </button>
                        </div>
                    </div>
                )}
            </div>
        </ErrorBoundary>
    );
};

export default ConvertForm;
