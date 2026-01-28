import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { useQueryClient } from '@tanstack/react-query';
import * as z from 'zod';
import { useWallets } from '../hooks/useWallets';
import { useBanks } from '../hooks/useBanks';
import { transactionApi } from '../api/transactions';
import { usersApi } from '../api/users';
import client from '../api/client';
import { ResolvedUser } from '@/types';
import { getErrorMessage } from '../utils/errorHandler';

const transferSchema = z.discriminatedUnion('transferType', [
    z.object({
        transferType: z.literal('internal'),
        amount: z.number().min(1).max(10000),
        currency: z.string().min(1),
        recipient: z.string().min(3, 'Username or email required'),
    }),
    z.object({
        transferType: z.literal('external'),
        amount: z.number().min(1).max(10000),
        currency: z.string().min(1),
        bankCode: z.string().min(1),
        accountNumber: z.string().length(10, '10 digits required'),
        accountName: z.string().min(1, 'Account name must be resolved'),
    }),
]);

type TransferFormValues = z.infer<typeof transferSchema>;

const TransferForm: React.FC = () => {
    const navigate = useNavigate();
    const queryClient = useQueryClient();
    const { data: wallets } = useWallets();
    const { data: banks } = useBanks();
    const [resolving, setResolving] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [submitting, setSubmitting] = useState(false);
    const [resolvedUser, setResolvedUser] = useState<ResolvedUser | null>(null);
    const [showConfirmation, setShowConfirmation] = useState(false);
    const [exchangeRate, setExchangeRate] = useState<number | null>(null);
    const [pendingExternalData, setPendingExternalData] = useState<any | null>(null);

    const {
        register,
        handleSubmit,
        watch,
        setValue,
    } = useForm<TransferFormValues>({
        resolver: zodResolver(transferSchema),
        defaultValues: { transferType: 'internal' }
    });

    const transferType = watch('transferType');
    const bankCode = (transferType === 'external') ? watch('bankCode') : '';
    const accountNumber = (transferType === 'external') ? watch('accountNumber') : '';
    const accountName = (transferType === 'external') ? watch('accountName') : '';

    useEffect(() => {
        if (transferType === 'external' && bankCode && accountNumber?.length === 10) {
            const resolve = async () => {
                setResolving(true);
                try {
                    const res = await client.get('/api/bank/resolve', { params: { bank_code: bankCode, account_number: accountNumber } });
                    setValue('accountName', res.data.account_name, { shouldValidate: true });
                } catch (err) {
                    setError(getErrorMessage(err));
                } finally {
                    setResolving(false);
                }
            };
            const timer = setTimeout(resolve, 500);
            return () => clearTimeout(timer);
        }
    }, [bankCode, accountNumber, transferType, setValue]);

    const onSubmit = async (data: TransferFormValues) => {
        console.log('Submitting transfer payload:', data);
        setError(null);

        if (data.transferType === 'internal') {
            setResolving(true);
            try {
                const user = await usersApi.resolveUser(data.recipient);
                setResolvedUser(user);
                setShowConfirmation(true);
            } catch (err) {
                setError(getErrorMessage(err));
            } finally {
                setResolving(false);
            }
            return;
        }

        // External transfer flow - 2 STEP
        setResolving(true);
        try {
            if (data.currency === 'NGN') {
                setExchangeRate(1);
            } else {
                const response = await transactionApi.getExchangeRate(data.currency, 'NGN');
                setExchangeRate(response.rate);
            }
            setPendingExternalData(data);
            setShowConfirmation(true);
        } catch (err: any) {
            setError(getErrorMessage(err));
        } finally {
            setResolving(false);
        }
    };

    const handleConfirmExternalTransfer = async () => {
        if (!pendingExternalData) return;
        setSubmitting(true);
        try {
            const result = await transactionApi.externalTransfer({
                amount: pendingExternalData.amount,
                currency: pendingExternalData.currency,
                bank_code: pendingExternalData.bankCode,
                account_number: pendingExternalData.accountNumber,
                account_name: pendingExternalData.accountName,
                reference: crypto.randomUUID(),
                idempotency_key: crypto.randomUUID(),
            });

            queryClient.invalidateQueries({ queryKey: ['wallets'] });
            queryClient.invalidateQueries({ queryKey: ['transactions'] });

            navigate(`/success?tx=${result.id || result.transaction_id || ''}`);
        } catch (err: any) {
            setError(getErrorMessage(err));
            setShowConfirmation(false);
        } finally {
            setSubmitting(false);
        }
    };

    const handleConfirmTransfer = async () => {
        if (!resolvedUser) return;
        const data = watch();

        setSubmitting(true);
        try {
            const result = await transactionApi.internalTransfer({
                recipient: resolvedUser.id, // Use resolved UUID
                amount: data.amount,
                currency: data.currency,
                reference: crypto.randomUUID(),
                idempotency_key: crypto.randomUUID(),
                description: 'Internal transfer'
            });

            // Invalidate queries to refresh dashboard data
            queryClient.invalidateQueries({ queryKey: ['wallets'] });
            queryClient.invalidateQueries({ queryKey: ['transactions'] });

            // Redirect to success page with transaction ID
            navigate(`/success?tx=${result.id || result.transaction_id || ''}`);
        } catch (err: any) {
            setError(getErrorMessage(err));
            setShowConfirmation(false);
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <div className="max-w-md mx-auto mt-4 sm:mt-10 p-6 sm:p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
            <div className="text-center mb-8">
                <div className="w-12 sm:w-16 h-12 sm:h-16 bg-gradient-to-r from-blue-500 to-indigo-500 rounded-2xl flex items-center justify-center mx-auto mb-4">
                    <span className="text-white text-xl sm:text-2xl">💸</span>
                </div>
                <h2 className="text-2xl sm:text-3xl font-bold text-gray-800 mb-2">Transfer Funds</h2>
            </div>

            <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
                <div>
                    <label className="input-label">Type</label>
                    <select {...register('transferType')} className="input-with-icon">
                        <option value="internal">Payego User</option>
                        <option value="external">Bank Account</option>
                    </select>
                </div>

                <div>
                    <label className="input-label">Currency</label>
                    <select {...register('currency')} className="input-with-icon">
                        <option value="">Select Currency</option>
                        {wallets?.map(w => (
                            <option key={w.currency} value={w.currency}>{w.currency} (Bal: {w.balance / 100})</option>
                        ))}
                    </select>
                </div>

                <div>
                    <label className="input-label">Amount</label>
                    <input type="number" step="0.01" {...register('amount', { valueAsNumber: true })} className="input-with-icon" />
                </div>

                {transferType === 'internal' && (
                    <div>
                        <label className="input-label">Recipient (Email or Username)</label>
                        <input type="text" {...register('recipient')} className="input-with-icon" placeholder="Enter username or email" />
                    </div>
                )}

                {transferType === 'external' && (
                    <>
                        <div>
                            <label className="input-label">Bank</label>
                            <select {...register('bankCode')} className="input-with-icon">
                                <option value="">Select Bank</option>
                                {banks?.map(b => <option key={b.code} value={b.code}>{b.name}</option>)}
                            </select>
                        </div>
                        <div>
                            <label className="input-label">Account Number</label>
                            <input type="text" {...register('accountNumber')} className="input-with-icon" />
                        </div>
                        <div className="p-3 bg-gray-50 rounded-lg text-sm">
                            {resolving ? 'Resolving...' : accountName ? <strong>{accountName}</strong> : 'Resolution pending...'}
                        </div>
                    </>
                )}

                <button type="submit" disabled={submitting || resolving} className="w-full btn-primary p-3 rounded-lg font-bold">
                    {submitting || resolving ? 'Processing...' : 'Send Money'}
                </button>
                {error && <p className="text-red-500 text-sm text-center">{error}</p>}
            </form>

            {showConfirmation && (resolvedUser || pendingExternalData) && (
                <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50 animate-in fade-in duration-200">
                    <div className="bg-white rounded-2xl p-6 sm:p-8 max-w-sm w-full shadow-2xl transform transition-all scale-100">
                        <div className="text-center mb-6">
                            <div className="w-16 h-16 bg-blue-50 text-blue-600 rounded-full flex items-center justify-center mx-auto mb-4">
                                <svg className="w-8 h-8" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                                    <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                                </svg>
                            </div>
                            <h3 className="text-xl font-bold text-gray-900">Confirm Transfer</h3>
                            <p className="text-gray-500 text-sm mt-1">Please review details</p>
                        </div>

                        <div className="space-y-4 bg-gray-50 rounded-xl p-4 mb-6">
                            <div className="flex justify-between items-center text-sm">
                                <span className="text-gray-500 font-medium">To:</span>
                                <div className="text-right">
                                    <p className="text-gray-900 font-bold leading-tight">
                                        {pendingExternalData ? pendingExternalData.accountName : (resolvedUser?.username || resolvedUser?.email)}
                                    </p>
                                    <p className="text-gray-500 text-xs">
                                        {pendingExternalData ? `${pendingExternalData.accountNumber}` : resolvedUser?.email}
                                    </p>
                                </div>
                            </div>

                            {pendingExternalData && (
                                <div className="flex justify-between items-center text-sm border-t border-gray-100 pt-3">
                                    <span className="text-gray-500 font-medium">Bank:</span>
                                    <span className="text-gray-900 font-bold">{banks?.find(b => b.code === pendingExternalData.bankCode)?.name || 'Unknown Bank'}</span>
                                </div>
                            )}

                            <div className="flex justify-between items-center text-sm border-t border-gray-100 pt-3">
                                <span className="text-gray-500 font-medium">Amount:</span>
                                <span className="text-gray-900 font-bold">{watch('currency')} {watch('amount')}</span>
                            </div>

                            {pendingExternalData && pendingExternalData.currency !== 'NGN' && exchangeRate && (
                                <div className="flex justify-between items-center text-sm border-t border-gray-100 pt-3">
                                    <span className="text-gray-500 font-medium">Amount in NGN:</span>
                                    <span className="text-green-600 font-bold">₦{(pendingExternalData.amount * exchangeRate).toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}</span>
                                </div>
                            )}
                        </div>

                        <div className="grid grid-cols-2 gap-3">
                            <button
                                onClick={() => {
                                    setShowConfirmation(false);
                                    setPendingExternalData(null);
                                    setResolvedUser(null);
                                }}
                                className="px-4 py-3 rounded-xl font-semibold text-gray-700 bg-gray-100 hover:bg-gray-200 transition-colors"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={pendingExternalData ? handleConfirmExternalTransfer : handleConfirmTransfer}
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

export default TransferForm;
