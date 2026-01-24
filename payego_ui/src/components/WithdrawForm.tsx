import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import { useWallets } from '../hooks/useWallets';
import { useUserBankAccounts } from '../hooks/useBanks';
import { transactionApi } from '../api/transactions';

const withdrawSchema = z.object({
    amount: z.number().min(1, 'Minimum 1 required'),
    currency: z.string().min(1, 'Select currency'),
    bankAccountId: z.string().min(1, 'Select bank account'),
});

type WithdrawFormValues = z.infer<typeof withdrawSchema>;

const WithdrawForm: React.FC = () => {
    const navigate = useNavigate();
    const { data: wallets } = useWallets();
    const { data: bankAccounts } = useUserBankAccounts();
    const [error, setError] = useState<string | null>(null);
    const [submitting, setSubmitting] = useState(false);

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
            await transactionApi.withdraw(data.bankAccountId, {
                amount: data.amount,
                currency: data.currency,
            });
            navigate('/dashboard');
        } catch (err: any) {
            setError(err.response?.data?.message || 'Withdrawal failed.');
        } finally {
            setSubmitting(false);
        }
    };

    return (
        <div className="max-w-md mx-auto mt-4 sm:mt-10 p-6 sm:p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
            <div className="text-center mb-8">
                <div className="w-12 sm:w-16 h-12 sm:h-16 bg-gradient-to-r from-purple-500 to-pink-500 rounded-2xl flex items-center justify-center mx-auto mb-4">
                    <span className="text-white text-xl sm:text-2xl">🏦</span>
                </div>
                <h2 className="text-2xl sm:text-3xl font-bold text-gray-800 mb-2">Withdraw Funds</h2>
            </div>

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

                <button type="submit" disabled={submitting} className="w-full btn-primary p-3 rounded-lg font-bold">
                    {submitting ? 'Processing...' : 'Withdraw'}
                </button>
                {error && <p className="text-red-500 text-sm text-center">{error}</p>}
            </form>
        </div>
    );
};

export default WithdrawForm;
