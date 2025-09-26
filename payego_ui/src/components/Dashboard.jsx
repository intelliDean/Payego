import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { Link } from 'react-router-dom';

function Dashboard() {
    const [user, setUser] = useState(null);
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(true);

    useEffect(() => {
        const fetchUser = async () => {
            try {
                const response = await axios.get(`${import.meta.env.VITE_API_URL}/api/current_user`, {
                    headers: {
                        'Authorization': `Bearer ${localStorage.getItem('jwt_token')}`,
                    },
                });
                setUser(response.data);
            } catch (err) {
                setError(err.response?.data?.message || 'Failed to fetch user data');
            } finally {
                setLoading(false);
            }
        };
        fetchUser();
    }, []);

    // Format balance based on currency
    const formatBalance = (balance, currency) => {
        return new Intl.NumberFormat('en-US', {
            style: 'currency',
            currency: currency,
            minimumFractionDigits: 2,
        }).format(balance / 100);
    };

    return (
        <div className="max-w-2xl mx-auto mt-10 p-6 bg-white rounded-lg shadow-md">
            <h2 className="text-2xl font-bold mb-6 text-center">Dashboard</h2>
            {loading && <p className="text-center">Loading...</p>}
            {error && <p className="text-red-500 text-center">{error}</p>}
            {user && (
                <div className="text-center">
                    <p className="text-lg mb-4"><strong>Email:</strong> {user.email}</p>
                    <h3 className="text-lg font-semibold mb-2">Wallet Balances</h3>
                    {user.wallets && user.wallets.length > 0 ? (
                        <ul className="mb-6">
                            {user.wallets.map((wallet) => (
                                <li key={wallet.currency} className="text-lg">
                                    <strong>{wallet.currency}:</strong> {formatBalance(wallet.balance, wallet.currency)}
                                </li>
                            ))}
                        </ul>
                    ) : (
                        <p className="text-lg mb-6">No wallets found.</p>
                    )}
                    <div className="grid grid-cols-2 gap-4">
                        <Link to="/top-up" className="bg-blue-600 text-white p-2 rounded hover:bg-blue-700 text-center">
                            Top Up
                        </Link>
                        <Link to="/add-bank" className="bg-blue-600 text-white p-2 rounded hover:bg-blue-700 text-center">
                            Manage Banks
                        </Link>
                        <Link to="/transfer" className="bg-blue-600 text-white p-2 rounded hover:bg-blue-700 text-center">
                            Transfer
                        </Link>
                        <Link to="/withdraw" className="bg-blue-600 text-white p-2 rounded hover:bg-blue-700 text-center">
                            Withdraw
                        </Link>
                        <Link to="/convert" className="bg-blue-600 text-white p-2 rounded hover:bg-blue-700 text-center">
                            Convert Currency
                        </Link>
                    </div>
                </div>
            )}
        </div>
    );
}

export default Dashboard;