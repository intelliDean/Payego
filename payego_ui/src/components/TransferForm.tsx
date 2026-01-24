import React, { useState, useEffect } from 'react';
import { useNavigate } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import { useWallets } from '../hooks/useWallets';
import { useBanks } from '../hooks/useBanks';
import { transactionApi } from '../api/transactions';
import client from '../api/client';

const transferSchema = z.discriminatedUnion('transferType', [
    z.object({
        transferType: z.literal('internal'),
        amount: z.number().min(1).max(10000),
        currency: z.string().min(1),
        recipientEmail: z.string().email('Valid email required'),
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
    const { data: wallets } = useWallets();
    const { data: banks } = useBanks();
    const [resolving, setResolving] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [submitting, setSubmitting] = useState(false);

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
                    const res = await client.get('/api/resolve_account', { params: { bank_code: bankCode, account_number: accountNumber } });
                    setValue('accountName', res.data.account_name, { shouldValidate: true });
                } catch (err) {
                    setError('Could not resolve account name.');
                } finally {
                    setResolving(false);
                }
            };
            const timer = setTimeout(resolve, 500);
            return () => clearTimeout(timer);
        }
    }, [bankCode, accountNumber, transferType, setValue]);

    const onSubmit = async (data: TransferFormValues) => {
        setSubmitting(true);
        setError(null);
        try {
            if (data.transferType === 'internal') {
                await transactionApi.internalTransfer({
                    amount: data.amount,
                    currency: data.currency,
                    recipient_email: data.recipientEmail
                });
            } else {
                await transactionApi.externalTransfer({
                    amount: data.amount,
                    currency: data.currency,
                    bank_code: data.bankCode,
                    account_number: data.accountNumber,
                    account_name: data.accountName
                });
            }
            navigate('/dashboard');
        } catch (err: any) {
            setError(err.response?.data?.message || 'Transfer failed.');
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
                        <label className="input-label">Recipient Email</label>
                        <input type="email" {...register('recipientEmail')} className="input-with-icon" />
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

                <button type="submit" disabled={submitting} className="w-full btn-primary p-3 rounded-lg font-bold">
                    {submitting ? 'Sending...' : 'Send Money'}
                </button>
                {error && <p className="text-red-500 text-sm text-center">{error}</p>}
            </form>
        </div>
    );
};

export default TransferForm;
