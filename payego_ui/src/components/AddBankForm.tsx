import React, { useState } from 'react';
import { useNavigate } from 'react-router-dom';
import { useForm } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import * as z from 'zod';
import { useBanks } from '../hooks/useBanks';
import { bankApi } from '../api/bank';

const bankAccountSchema = z.object({
    bankCode: z.string().min(1, 'Pick a bank, don’t be shy!'),
    accountNumber: z.string().regex(/^\d{10}$/, 'Account number needs 10 digits.'),
});

type BankAccountFormValues = z.infer<typeof bankAccountSchema>;

const AddBankForm: React.FC = () => {
    const navigate = useNavigate();
    const { data: banks, isLoading: fetching, error: fetchError } = useBanks();
    const [error, setError] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);

    const {
        register,
        handleSubmit,
        formState: { errors },
    } = useForm<BankAccountFormValues>({
        resolver: zodResolver(bankAccountSchema),
    });

    const onSubmit = async (data: BankAccountFormValues) => {
        setLoading(true);
        setError(null);

        const selectedBankData = banks?.find(bank => bank.code === data.bankCode);
        if (!selectedBankData) {
            setError('That bank’s lost in space!');
            setLoading(false);
            return;
        }

        try {
            await bankApi.addBankAccount({
                account_number: data.accountNumber,
                bank_code: data.bankCode,
                bank_name: selectedBankData.name,
            });
            navigate('/banks');
        } catch (err: any) {
            setError(err.response?.data?.message || 'Failed to add bank account.');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="max-w-md mx-auto mt-4 sm:mt-10 p-6 sm:p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
            <div className="text-center mb-8">
                <div className="w-12 sm:w-16 h-12 sm:h-16 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-2xl flex items-center justify-center mx-auto mb-4">
                    <span className="text-white text-xl sm:text-2xl">🏛️</span>
                </div>
                <h2 className="text-2xl sm:text-3xl font-bold text-gray-800 mb-2">Add Bank Account</h2>
                <p className="text-gray-600 text-sm sm:text-base">Connect your bank account for withdrawals</p>
            </div>

            {fetching ? (
                <div className="flex flex-col items-center justify-center py-12">
                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                    <p className="mt-4 text-gray-600">Fetching banks...</p>
                </div>
            ) : (
                <form onSubmit={handleSubmit(onSubmit)} className="space-y-4">
                    <div>
                        <label className="block text-gray-700 font-medium mb-1 text-sm">Bank Name</label>
                        <select
                            {...register('bankCode')}
                            className={`w-full p-3 border rounded-lg focus:ring-2 focus:ring-blue-500 ${errors.bankCode ? 'border-red-500' : 'border-gray-300'}`}
                        >
                            <option value="">Select a bank</option>
                            {banks?.map(bank => (
                                <option key={bank.code} value={bank.code}>{bank.name}</option>
                            ))}
                        </select>
                        {errors.bankCode && <p className="mt-1 text-xs text-red-500">{errors.bankCode.message}</p>}
                    </div>

                    <div>
                        <label className="block text-gray-700 font-medium mb-1 text-sm">Account Number</label>
                        <input
                            type="text"
                            {...register('accountNumber')}
                            className={`w-full p-3 border rounded-lg focus:ring-2 focus:ring-blue-500 ${errors.accountNumber ? 'border-red-500' : 'border-gray-300'}`}
                            placeholder="Enter 10-digit account number"
                        />
                        {errors.accountNumber && <p className="mt-1 text-xs text-red-500">{errors.accountNumber.message}</p>}
                    </div>

                    <div className="flex space-x-4 pt-2">
                        <button
                            type="submit"
                            disabled={loading}
                            className="flex-1 btn-primary text-white p-3 rounded-lg font-medium shadow-md hover:shadow-lg transform hover:-translate-y-0.5 transition-all"
                        >
                            {loading ? 'Adding...' : 'Add Account'}
                        </button>
                        <button
                            type="button"
                            onClick={() => navigate('/banks')}
                            className="flex-1 bg-gray-100 text-gray-700 p-3 rounded-lg hover:bg-gray-200 transition-all font-medium"
                        >
                            Cancel
                        </button>
                    </div>

                    {(error || fetchError) && (
                        <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg text-center">
                            <p className="text-red-600 text-sm">{error || 'Failed to load banks.'}</p>
                        </div>
                    )}
                </form>
            )}
        </div>
    );
};

export default AddBankForm;
