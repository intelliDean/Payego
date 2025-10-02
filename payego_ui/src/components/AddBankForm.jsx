import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { useNavigate } from 'react-router-dom';

function AddBankForm() {
    const [accountNumber, setAccountNumber] = useState('');
    const [selectedBank, setSelectedBank] = useState('');
    const [banks, setBanks] = useState([]);
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(false);
    const [fetching, setFetching] = useState(true);
    const navigate = useNavigate();

    useEffect(() => {
        const fetchBanks = async () => {
            try {
                setFetching(true);
                const token = localStorage.getItem('jwt_token') || sessionStorage.getItem('jwt_token');
                if (!token) {
                    setError('No session found. Log in to continue!');
                    navigate('/login');
                    return;
                }

                const response = await axios.get(`${import.meta.env.VITE_API_URL}/api/banks`, {
                    headers: { 'Authorization': `Bearer ${token}` },
                });
                setBanks(response.data.banks);
                if (response.data.banks.length > 0) {
                    setSelectedBank(response.data.banks[0].code); // Default to first bank
                }
            } catch (err) {
                if (err.response?.status === 401) {
                    setError('Session expired. Log in again!');
                    localStorage.removeItem('jwt_token');
                    sessionStorage.removeItem('jwt_token');
                    navigate('/login');
                } else {
                    setError(err.response?.data?.message || 'Banks vanished! Try again!');
                }
            } finally {
                setFetching(false);
            }
        };
        fetchBanks();
    }, [navigate]);

    const handleSubmit = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);

        // Client-side validation
        if (!selectedBank) {
            setError('Pick a bank, don’t be shy!');
            setLoading(false);
            return;
        }
        if (!accountNumber) {
            setError('Account number’s missing. Where’s it hiding?');
            setLoading(false);
            return;
        }
        if (!/^\d{10}$/.test(accountNumber)) {
            setError('Account number needs 10 digits. No funny business!');
            setLoading(false);
            return;
        }

        const selectedBankData = banks.find(bank => bank.code === selectedBank); // Fixed typo
        if (!selectedBankData) {
            setError('That bank’s lost in space!');
            setLoading(false);
            return;
        }

        try {
            const token = localStorage.getItem('jwt_token') || sessionStorage.getItem('jwt_token');
            await axios.post(
                `${import.meta.env.VITE_API_URL}/api/add_bank`,
                {
                    account_number: accountNumber,
                    bank_code: selectedBank,
                    bank_name: selectedBankData.name,
                },
                {
                    headers: {
                        'Content-Type': 'application/json',
                        'Authorization': `Bearer ${token}`,
                    },
                }
            );
            navigate('/banks');
        } catch (err) {
            const message = err.response?.data?.message;
            if (message === 'Invalid account number') {
                setError('That account number’s playing hard to get!');
            } else if (message === 'Bank account already linked') {
                setError('This bank account’s already part of the Payego party!');
            } else {
                setError(message || 'Bank account ran away! Try again!');
            }
        } finally {
            setLoading(false);
        }
    };

    const handleCancel = () => {
        navigate('/banks');
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

            {fetching && (
                <div className="flex flex-col items-center justify-center py-12">
                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                    <p className="mt-4 text-gray-600">Fetching banks...</p>
                </div>
            )}

            {!fetching && (
                <form onSubmit={handleSubmit}>
                    <div className="mb-4">
                        <label htmlFor="bank-select" className="block text-gray-700 font-medium mb-2">
                            Bank Name
                        </label>
                        <select
                            id="bank-select"
                            value={selectedBank}
                            onChange={(e) => setSelectedBank(e.target.value)}
                            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                            required
                            aria-describedby={error && !selectedBank ? 'bank-error' : undefined}
                        >
                            <option value="" disabled>Select a bank</option>
                            {banks.map(bank => (
                                <option key={bank.code} value={bank.code}>
                                    {bank.name}
                                </option>
                            ))}
                        </select>
                    </div>
                    <div className="mb-4">
                        <label htmlFor="account-number" className="block text-gray-700 font-medium mb-2">
                            Account Number
                        </label>
                        <input
                            id="account-number"
                            type="text"
                            value={accountNumber}
                            onChange={(e) => {
                                const value = e.target.value.replace(/\D/g, ''); // Allow only digits
                                setAccountNumber(value.slice(0, 10)); // Limit to 10 digits
                            }}
                            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                            placeholder="Enter 10-digit account number"
                            required
                            aria-describedby={error && accountNumber ? 'account-number-error' : undefined}
                        />
                    </div>
                    <div className="flex space-x-4">
                        <button
                            type="submit"
                            disabled={loading || banks.length === 0}
                            className="flex-1 bg-gradient-to-r from-blue-600 to-indigo-600 text-white p-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 disabled:from-gray-400 disabled:to-gray-400 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                        >
                            {loading ? 'Adding...' : 'Add Bank Account'}
                        </button>
                        <button
                            type="button"
                            onClick={handleCancel}
                            className="flex-1 bg-gray-200 text-gray-700 p-3 rounded-lg hover:bg-gray-300 transition-all duration-200 font-medium"
                        >
                            Cancel
                        </button>
                    </div>
                    {error && (
                        <div id="error-message" className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                            <p className="text-red-600 text-center text-sm">{error}</p>
                        </div>
                    )}
                </form>
            )}
        </div>
    );
}

export default AddBankForm;



