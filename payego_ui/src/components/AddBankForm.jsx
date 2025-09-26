import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { useNavigate } from 'react-router-dom';

function AddBankForm() {
    const [accountNumber, setAccountNumber] = useState('');
    const [selectedBank, setSelectedBank] = useState('');
    const [banks, setBanks] = useState([]);
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(false);
    const navigate = useNavigate();

    useEffect(() => {
        const fetchBanks = async () => {
            try {
                const response = await axios.get(`${import.meta.env.VITE_API_URL}/api/banks`);
                console.log("Banks: ", response.data);
                setBanks(response.data.banks);
                if (response.data.banks.length > 0) {
                    setSelectedBank(response.data.banks[0].code); // Default to first bank
                }
            } catch (err) {
                setError(err.response?.data?.message || 'Failed to fetch banks');
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
        <div className="max-w-md mx-auto mt-10 p-6 bg-white rounded-lg shadow-md">
            <h2 className="text-2xl font-bold mb-6 text-center">Add Bank Account</h2>
            <form onSubmit={handleSubmit}>
                <div className="mb-4">
                    <label className="block text-gray-700">Bank Name</label>
                    <select
                        value={selectedBank}
                        onChange={(e) => setSelectedBank(e.target.value)}
                        className="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
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
                    <label className="block text-gray-700">Account Number</label>
                    <input
                        type="text"
                        value={accountNumber}
                        onChange={(e) => setAccountNumber(e.target.value)}
                        className="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                        required
                    />
                </div>
                <button
                    type="submit"
                    disabled={loading || banks.length === 0}
                    className="w-full bg-blue-600 text-white p-2 rounded hover:bg-blue-700 disabled:bg-blue-300"
                >
                    {loading ? 'Adding...' : 'Add Bank Account'}
                </button>
                {error && <p className="text-red-500 mt-4 text-center">{error}</p>}
            </form>
        </div>
    );
}

export default AddBankForm;