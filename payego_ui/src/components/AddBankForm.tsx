import React, { useState } from 'react';
import { useQueryClient } from '@tanstack/react-query';
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
    const queryClient = useQueryClient();
    const { data: banks, isLoading: fetching, error: fetchError } = useBanks();
    const [error, setError] = useState<string | null>(null);
    const [loading, setLoading] = useState(false);
    const [showConfirmation, setShowConfirmation] = useState(false);
    const [showSuccess, setShowSuccess] = useState(false);
    const [pendingData, setPendingData] = useState<BankAccountFormValues | null>(null);
    const [resolvedAccountName, setResolvedAccountName] = useState<string>('');

    const {
        register,
        handleSubmit,
        watch,
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

    const onPreview = async (data: BankAccountFormValues) => {
        setLoading(true);
        setError(null);

        const selectedBankData = banks?.find(bank => bank.code === data.bankCode);
        if (!selectedBankData) {
            setError('That bank\'s lost in space!');
            setLoading(false);
            return;
        }

        try {
            // Resolve account name
            const response = await bankApi.resolveAccount(data.bankCode, data.accountNumber);
            setResolvedAccountName(response.account_name);
            setPendingData(data);
            setShowConfirmation(true);
        } catch (err: any) {
            setError(err.response?.data?.message || 'Failed to verify account.');
        } finally {
            setLoading(false);
        }


    };

    const onConfirm = async () => {
        if (!pendingData) return;

        setLoading(true);
        setError(null);

        const selectedBankData = banks?.find(bank => bank.code === pendingData.bankCode);
        if (!selectedBankData) {
            setError('That bank\'s lost in space!');
            setLoading(false);
            return;
        }

        try {
            await bankApi.addBankAccount({
                account_number: pendingData.accountNumber,
                bank_code: pendingData.bankCode,
                bank_name: selectedBankData.name,
            });
            setShowConfirmation(false);

            // Invalidate queries to refresh bank list
            queryClient.invalidateQueries({ queryKey: ['bankAccounts'] });

            setShowSuccess(true);
        } catch (err: any) {
            setError(err.response?.data?.message || 'Failed to add bank account.');
            setShowConfirmation(false);
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
                <form onSubmit={handleSubmit(onPreview)} className="space-y-4">
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
                            {loading ? 'Verifying...' : 'Add Account'}
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

            {/* Confirmation Modal */}
            {showConfirmation && pendingData && (
                <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
                    <div className="bg-white rounded-xl p-6 max-w-sm w-full shadow-2xl">
                        <h3 className="text-xl font-bold mb-4">Confirm Bank Account</h3>
                        <div className="space-y-3 mb-6">
                            <div className="flex justify-between">
                                <span className="text-gray-600">Bank:</span>
                                <span className="font-semibold">{banks?.find(b => b.code === pendingData.bankCode)?.name}</span>
                            </div>
                            <div className="flex justify-between">
                                <span className="text-gray-600">Account Number:</span>
                                <span className="font-semibold">{pendingData.accountNumber}</span>
                            </div>
                            <div className="flex justify-between">
                                <span className="text-gray-600">Account Name:</span>
                                <span className="font-semibold text-green-600">{resolvedAccountName}</span>
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
                                disabled={loading}
                                className="flex-1 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 disabled:opacity-50"
                            >
                                {loading ? 'Adding...' : 'Confirm'}
                            </button>
                        </div>
                    </div>
                </div>
            )}

            {/* Success Modal */}
            {showSuccess && (
                <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center p-4 z-50">
                    <div className="bg-white rounded-xl p-6 max-w-sm w-full shadow-2xl text-center">
                        <div className="w-16 h-16 bg-gradient-to-r from-green-500 to-emerald-500 rounded-full flex items-center justify-center mx-auto mb-4">
                            <svg className="w-8 h-8 text-white" viewBox="0 0 24 24" fill="currentColor">
                                <path d="M20.285 2l-11.285 11.567-5.286-5.011-3.714 3.716 9 8.728 15-15.285z" />
                            </svg>
                        </div>
                        <h3 className="text-2xl font-bold text-gray-900 mb-2">Bank Account Added!</h3>
                        <p className="text-gray-600 mb-6">Your bank account has been successfully linked</p>
                        <div className="bg-gray-50 rounded-lg p-4 mb-6 space-y-2 text-left">
                            <div className="flex justify-between text-sm">
                                <span className="text-gray-600">Bank:</span>
                                <span className="font-semibold">{banks?.find(b => b.code === pendingData?.bankCode)?.name}</span>
                            </div>
                            <div className="flex justify-between text-sm">
                                <span className="text-gray-600">Account:</span>
                                <span className="font-semibold">{pendingData?.accountNumber}</span>
                            </div>
                            <div className="flex justify-between text-sm">
                                <span className="text-gray-600">Name:</span>
                                <span className="font-semibold">{resolvedAccountName}</span>
                            </div>
                        </div>
                        <button
                            onClick={() => navigate('/banks')}
                            className="w-full bg-gradient-to-r from-blue-600 to-indigo-600 text-white px-6 py-3 rounded-lg font-medium shadow-lg hover:shadow-xl"
                        >
                            View My Banks
                        </button>
                    </div>
                </div>
            )}
        </div>
    );
};

export default AddBankForm;
