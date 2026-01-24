import React, { useState } from "react";
import { useNavigate } from "react-router-dom";
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import ErrorBoundary from "./ErrorBoundary";
import { useWallets } from "../hooks/useWallets";
import { transactionApi } from "../api/transactions";
import { Currency } from "../types";

const convertSchema = z.object({
    amount: z.number().min(0.01).max(10000),
    fromCurrency: z.string().min(1),
    toCurrency: z.string().min(1),
}).refine(data => data.fromCurrency !== data.toCurrency, {
    message: "Cannot convert to same currency",
    path: ["toCurrency"]
});

type ConvertFormValues = z.infer<typeof convertSchema>;

const SUPPORTED_CURRENCIES: Currency[] = ['USD', 'EUR', 'GBP', 'NGN'];

const ConvertForm: React.FC = () => {
    const navigate = useNavigate();
    const { data: wallets, isLoading: fetching } = useWallets();
    const [submitting, setSubmitting] = useState(false);
    const [error, setError] = useState<string | null>(null);

    const {
        register,
        handleSubmit,
        formState: { errors },
    } = useForm<ConvertFormValues>({
        resolver: zodResolver(convertSchema),
    });

    const onSubmit = async (data: ConvertFormValues) => {
        setSubmitting(true);
        setError(null);
        try {
            await transactionApi.convertCurrency({
                amount: data.amount,
                from_currency: data.fromCurrency,
                to_currency: data.toCurrency
            });
            navigate("/dashboard");
        } catch (err: any) {
            setError(err.response?.data?.message || "Conversion failed.");
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
                    <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
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
                                {submitting ? 'Converting...' : 'Convert'}
                            </button>
                            <button type="button" onClick={() => navigate('/dashboard')} className="flex-1 bg-gray-100 p-3 rounded-lg font-bold">Cancel</button>
                        </div>

                        {error && <div className="alert-error text-center text-sm">{error}</div>}
                    </form>
                )}
            </div>
        </ErrorBoundary>
    );
};

export default ConvertForm;
