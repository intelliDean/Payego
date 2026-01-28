import React, { useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
import { useNavigate } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import { useWallets } from '../hooks/useWallets';
import { useUserBankAccounts } from '../hooks/useBanks';
import { transactionApi } from '../api/transactions';
import { getErrorMessage } from '../utils/errorHandler';
import { useAuth } from '../contexts/AuthContext';
import { Link } from 'react-router-dom';

const withdrawSchema = z.object({
    amount: z.number().min(1, 'Minimum 1 required'),
    currency: z.string().min(1, 'Select currency'),
    bankAccountId: z.string().min(1, 'Select bank account'),
});

type WithdrawFormValues = z.infer<typeof withdrawSchema>;

const WithdrawForm: React.FC = () => {
    const navigate = useNavigate();
    const queryClient = useQueryClient();
    const { user } = useAuth();
    const { data: wallets } = useWallets();
    const { data: bankAccounts } = useUserBankAccounts();
    const [error, setError] = useState<string | null>(null);
    const [submitting, setSubmitting] = useState(false);
    const [showConfirmation, setShowConfirmation] = useState(false);
    const [exchangeRate, setExchangeRate] = useState<number | null>(null);
    const [pendingData, setPendingData] = useState<WithdrawFormValues | null>(null);

    const {
        register,
        handleSubmit,
        formState: { errors },
    } = useForm<WithdrawFormValues>({
        resolver: zodResolver(withdrawSchema),
    });

    const onSubmit = async (data: WithdrawFormValues) => {
        setSubmitting(true);
        setError(null);
        try {
            if (data.currency === 'NGN') {
                setExchangeRate(1);
            } else {
                const response = await transactionApi.getExchangeRate(data.currency, 'NGN');
                setExchangeRate(response.rate);
            }
            setPendingData(data);
            setShowConfirmation(true);
        } catch (err: any) {
            setError(getErrorMessage(err));
        } finally {
            setSubmitting(false);
        }
    };

    const onConfirm = async () => {
        if (!pendingData) return;
        setSubmitting(true);
        setError(null);
        try {
            // Generate mandatory fields for backend validation
            const reference = crypto.randomUUID();
            const idempotencyKey = crypto.randomUUID();

            const res = await transactionApi.withdraw(pendingData.bankAccountId, {
                amount: pendingData.amount,
                currency: pendingData.currency,
                reference: reference,
                idempotency_key: idempotencyKey,
            });
            setShowConfirmation(false);

            // Invalidate queries to refresh balance and history
            queryClient.invalidateQueries({ queryKey: ['wallets'] });
            queryClient.invalidateQueries({ queryKey: ['transactions'] });

            navigate(`/success?tx=${res.transaction_id}`);
        } catch (err: any) {
            setError(getErrorMessage(err));
            setShowConfirmation(false);
        } finally {
            setSubmitting(false);
        }
    };

    const selectedBank = bankAccounts?.find(b => b.id === pendingData?.bankAccountId);

    return (
        <div className="max-w-md mx-auto mt-4 sm:mt-10 p-6 sm:p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
            <div className="text-center mb-8">
                <div className="w-12 sm:w-16 h-12 sm:h-16 bg-gradient-to-r from-purple-500 to-pink-500 rounded-2xl flex items-center justify-center mx-auto mb-4">
                    <span className="text-white text-xl sm:text-2xl">🏦</span>
                </div>
                <h2 className="text-2xl sm:text-3xl font-bold text-gray-800 mb-2">Withdraw Funds</h2>
            </div>

            {!user?.email_verified_at && (
                <div className="mb-6 p-4 bg-yellow-50 border-l-4 border-yellow-400 rounded-r-xl">
                    <div className="flex">
                        <div className="flex-shrink-0">
                            <span className="text-yellow-400">⚠️</span>
                        </div>
                        <div className="ml-3">
                            <p className="text-sm text-yellow-700 font-bold">
                                Email Verification Required
                            </p>
                            <p className="text-xs text-yellow-600 mt-1">
                                Please verify your email to unlock withdrawals. <Link to="/security" className="underline font-black">Go to Security</Link>
                            </p>
                        </div>
                    </div>
                </div>
            )}

            <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
                <div>
                    <label className="input-label">Currency</label>
                    <select {...register('currency')} className="input-with-icon">
                        <option value="">Select Currency</option>
                        {wallets?.map(w => (
                            <option key={w.currency} value={w.currency}>{w.currency} (Bal: {w.balance / 100})</option>
                        ))}
                    </select>
                    {errors.currency && <p className="text-red-500 text-xs mt-1">{errors.currency.message}</p>}
                </div>

                <div>
                    <label className="input-label">Amount</label>
                    <input type="number" step="0.01" {...register('amount', { valueAsNumber: true })} className="input-with-icon" />
                    {errors.amount && <p className="text-red-500 text-xs mt-1">{errors.amount.message}</p>}
                </div>

                <div>
                    <label className="input-label">Transfer to Bank Account</label>
                    <select {...register('bankAccountId')} className="input-with-icon">
                        <option value="">Select Bank Account</option>
                        {bankAccounts?.map(b => (
                            <option key={b.id} value={b.id}>{b.bank_name} - {b.account_number}</option>
                        ))}
                    </select>
                    {errors.bankAccountId && <p className="text-red-500 text-xs mt-1">{errors.bankAccountId.message}</p>}
                </div>

                <button
                    type="submit"
                    disabled={submitting || !user?.email_verified_at}
                    className={`w-full btn-primary p-3 rounded-lg font-bold ${!user?.email_verified_at ? 'opacity-50 cursor-not-allowed grayscale' : ''}`}
                >
                    {submitting ? 'Processing...' : 'Withdraw'}
                </button>
                {error && <p className="text-red-500 text-sm text-center">{error}</p>}
            </form>

            {/* Confirmation Modal */}
            {showConfirmation && pendingData && (
                <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50 animate-in fade-in duration-200">
                    <div className="bg-white rounded-2xl p-6 sm:p-8 max-w-sm w-full shadow-2xl transform transition-all scale-100">
                        <div className="text-center mb-6">
                            <div className="w-16 h-16 bg-blue-50 text-blue-600 rounded-full flex items-center justify-center mx-auto mb-4">
                                <svg className="w-8 h-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                            </div>
                            <h3 className="text-xl font-bold text-gray-900">Confirm Withdrawal</h3>
                            <p className="text-gray-500 text-sm mt-1">Please review your withdrawal details</p>
                        </div>

                        <div className="space-y-4 bg-gray-50 rounded-xl p-4 mb-6">
                            <div className="flex justify-between items-center text-sm">
                                <span className="text-gray-500 font-medium">Withdrawal Amount:</span>
                                <span className="text-gray-900 font-bold">{pendingData.amount} {pendingData.currency}</span>
                            </div>

                            {pendingData.currency !== 'NGN' && exchangeRate && (
                                <div className="flex justify-between items-center text-sm border-t border-gray-200 pt-3">
                                    <span className="text-gray-500 font-medium">Amount in NGN:</span>
                                    <span className="text-green-600 font-bold">₦{(pendingData.amount * exchangeRate).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
                                </div>
                            )}

                            <div className="flex justify-between items-center text-sm border-t border-gray-200 pt-3">
                                <span className="text-gray-500 font-medium">Bank Account:</span>
                                <div className="text-right">
                                    <p className="text-gray-900 font-bold leading-tight">{selectedBank?.bank_name}</p>
                                    <p className="text-gray-500 text-xs">{selectedBank?.account_number}</p>
                                </div>
                            </div>
                        </div>

                        <div className="grid grid-cols-2 gap-3">
                            <button
                                onClick={() => setShowConfirmation(false)}
                                className="px-4 py-3 rounded-xl font-semibold text-gray-700 bg-gray-100 hover:bg-gray-200 transition-colors"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={onConfirm}
                                disabled={submitting}
                                className="px-4 py-3 rounded-xl font-semibold text-white bg-gradient-to-r from-blue-600 to-indigo-600 hover:shadow-lg transition-all disabled:opacity-50"
                            >
                                {submitting ? 'Processing...' : 'Confirm'}
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
};

export default WithdrawForm;
