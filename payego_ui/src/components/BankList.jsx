import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { Link } from 'react-router-dom';

function BankList() {
    const [banks, setBanks] = useState([]);
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchBanks = async () => {
            try {
                const response = await axios.get(`${import.meta.env.VITE_API_URL}/api/banks`);
                setBanks(response.data);
            } catch (err) {
                setError(err.response?.data?.message || 'Failed to fetch banks');
            } finally {
                setLoading(false);
            }
        };
        fetchBanks();
    }, []);

    const handleInitBanks = async () => {
        setLoading(true);
        setError(null);
        try {
            await axios.post(`${import.meta.env.VITE_API_URL}/api/bank/init`, {}, {
                headers: {
                    'Content-Type': 'application/json',
                    'Authorization': `Bearer ${localStorage.getItem('jwt_token')}`,
                },
            });
            const response = await axios.get(`${import.meta.env.VITE_API_URL}/api/banks`);
            setBanks(response.data);
        } catch (err) {
            setError(err.response?.data?.message || 'Failed to initialize banks');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="max-w-2xl mx-auto mt-10 p-6 bg-white rounded-lg shadow-md">
            <h2 className="text-2xl font-bold mb-6 text-center">Available Banks</h2>
            <button
                onClick={handleInitBanks}
                disabled={loading}
                className="mb-4 bg-green-600 text-white p-2 rounded hover:bg-green-700 disabled:bg-green-300"
            >
                {loading ? 'Initializing...' : 'Initialize Banks'}
            </button>
            {loading && <p className="text-center">Loading...</p>}
            {error && <p className="text-red-500 text-center">{error}</p>}
            <ul className="space-y-4">
                {banks.map((bank) => (
                    <li key={bank.id} className="p-4 border rounded">
                        <p><strong>Code:</strong> {bank.code}</p>
                        <p><strong>Name:</strong> {bank.name}</p>
                    </li>
                ))}
            </ul>
            <p className="mt-4 text-center">
                <Link to="/add-bank" className="text-blue-600 hover:underline">Add Bank Account</Link>
            </p>
        </div>
    );
}

export default BankList;