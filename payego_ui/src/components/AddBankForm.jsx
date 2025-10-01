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
                const response = await axios.get(`${import.meta.env.VITE_API_URL}/api/banks`);
                console.log("Banks: ", response.data);
                setBanks(response.data.banks);
                if (response.data.banks.length > 0) {
                    setSelectedBank(response.data.banks[0].code); // Default to first bank
                }
            } catch (err) {
                setError(err.response?.data?.message || 'Failed to fetch banks');
            } finally {
                setFetching(false);
            }
        };
        fetchBanks();
    }, []);

    const handleSubmit = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);

        const selectedBankData = banks.find(bank => bank.code === selectedBank);
        if (!selectedBankData) {
            setError('Please select a valid bank');
            setLoading(false);
            return;
        }

        try {
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
                        'Authorization': `Bearer ${localStorage.getItem('jwt_token')}`,
                    },
                }
            );
            navigate('/banks');
        } catch (err) {
            setError(err.response?.data?.message || 'Failed to add bank account');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="max-w-md mx-auto mt-10 p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
            <div className="text-center mb-8">
                <div className="w-16 h-16 bg-gradient-to-r from-teal-500 to-cyan-500 rounded-2xl flex items-center justify-center mx-auto mb-4">
                    <span className="text-white text-2xl">🏛️</span>
                </div>
                <h2 className="text-3xl font-bold text-gray-800 mb-2">Add Bank Account</h2>
                <p className="text-gray-600">Connect your bank account for withdrawals</p>
            </div>
            
            {fetching && (
                <div className="flex justify-center items-center py-12">
                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-teal-600"></div>
                </div>
            )}
            
            {!fetching && (
            <form onSubmit={handleSubmit}>
                <div className="mb-4">
                    <label className="block text-gray-700 font-medium mb-2">Bank Name</label>
                    <select
                        value={selectedBank}
                        onChange={(e) => setSelectedBank(e.target.value)}
                        className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-teal-500 focus:border-transparent transition-all duration-200"
                        required
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
                    <label className="block text-gray-700 font-medium mb-2">Account Number</label>
                    <input
                        type="text"
                        value={accountNumber}
                        onChange={(e) => setAccountNumber(e.target.value)}
                        className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-teal-500 focus:border-transparent transition-all duration-200"
                        placeholder="Enter your account number"
                        required
                    />
                </div>
                <button
                    type="submit"
                    disabled={loading || banks.length === 0}
                    className="w-full bg-gradient-to-r from-teal-500 to-cyan-500 text-white p-3 rounded-lg hover:from-teal-600 hover:to-cyan-600 disabled:from-gray-400 disabled:to-gray-400 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                >
                    {loading ? 'Adding...' : 'Add Bank Account'}
                </button>
                {error && (
                    <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                        <p className="text-red-600 text-center text-sm">{error}</p>
                    </div>
                )}
            </form>
            )}
        </div>
    );
}

export default AddBankForm;