import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { useNavigate } from 'react-router-dom';

function ConvertForm() {
    const [amount, setAmount] = useState('');
    const [fromCurrency, setFromCurrency] = useState('');
    const [toCurrency, setToCurrency] = useState('');
    const [wallets, setWallets] = useState([]);
    const [supportedCurrencies, setSupportedCurrencies] = useState([]);
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(false);
    const [fetching, setFetching] = useState(true);
    const navigate = useNavigate();

    useEffect(() => {
        const fetchData = async () => {
            try {
                setFetching(true);
                const walletsResponse = await axios.get(`${import.meta.env.VITE_API_URL}/api/wallets`, {
                    headers: { 'Authorization': `Bearer ${localStorage.getItem('jwt_token')}` },
                });

                setWallets(walletsResponse.data.wallets || []);
                if (walletsResponse.data.wallets.length > 0) {
                    setFromCurrency(walletsResponse.data.wallets[0].currency);
                }

                // Hardcode supported currencies
                setSupportedCurrencies([
                    'USD', 'NGN', 'GBP', 'EUR', 'CAD', 'AUD', 'JPY', 'CHF', 'CNY',
                    'SEK', 'NZD', 'MXN', 'SGD', 'HKD', 'NOK', 'KRW', 'TRY', 'INR', 'BRL', 'ZAR'
                ]);
                if (supportedCurrencies.length > 0) {
                    setToCurrency(supportedCurrencies[0]);
                }
            } catch (err) {
                setError(err.response?.data?.message || 'Failed to fetch wallets');
            } finally {
                setFetching(false);
            }
        };
        fetchData();
    }, []);

    const handleSubmit = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);

        if (fromCurrency === toCurrency) {
            setError('From and to currencies must be different');
            setLoading(false);
            return;
        }
        if (!amount || amount < 1 || amount > 10000) {
            setError('Amount must be between 1 and 10,000');
            setLoading(false);
            return;
        }
        const selectedWallet = wallets.find(w => w.currency === fromCurrency);
        if (!selectedWallet) {
            setError(`No wallet found for ${fromCurrency}`);
            setLoading(false);
            return;
        }
        if ((amount * 100) > selectedWallet.balance) {
            setError(`Insufficient balance: available ${(selectedWallet.balance / 100).toFixed(2)} ${fromCurrency}`);
            setLoading(false);
            return;
        }

        try {
            const response = await axios.post(
                `${import.meta.env.VITE_API_URL}/api/convert_currency`,
                {
                    amount: parseFloat(amount),
                    from_currency: fromCurrency,
                    to_currency: toCurrency,
                },
                {
                    headers: {
                        'Content-Type': 'application/json',
                        'Authorization': `Bearer ${localStorage.getItem('jwt_token')}`,
                    },
                }
            );
            alert(`Conversion completed! Transaction ID: ${response.data.transaction_id}, Converted: ${response.data.converted_amount} ${toCurrency}`);
            navigate('/dashboard');
        } catch (err) {
            setError(err.response?.data?.message || 'Failed to process conversion');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="max-w-md mx-auto mt-10 p-6 bg-white rounded-lg shadow-md">
            <h2 className="text-2xl font-bold mb-6 text-center">Convert Currency</h2>
            {fetching ? (
                <p className="text-center">Loading...</p>
            ) : (
                <form onSubmit={handleSubmit}>
                    <div className="mb-4">
                        <label className="block text-gray-700">From Currency</label>
                        <select
                            value={fromCurrency}
                            onChange={(e) => setFromCurrency(e.target.value)}
                            className="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                            required
                        >
                            <option value="" disabled>Select from currency</option>
                            {wallets.map(wallet => (
                                <option key={wallet.currency} value={wallet.currency}>
                                    {wallet.currency} (Balance: {(wallet.balance / 100).toFixed(2)})
                                </option>
                            ))}
                        </select>
                    </div>
                    <div className="mb-4">
                        <label className="block text-gray-700">To Currency</label>
                        <select
                            value={toCurrency}
                            onChange={(e) => setToCurrency(e.target.value)}
                            className="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                            required
                        >
                            <option value="" disabled>Select to currency</option>
                            {supportedCurrencies.map(curr => (
                                <option key={curr} value={curr}>
                                    {curr}
                                </option>
                            ))}
                        </select>
                    </div>
                    <div className="mb-4">
                        <label className="block text-gray-700">Amount</label>
                        <input
                            type="number"
                            value={amount}
                            onChange={(e) => setAmount(e.target.value)}
                            min="1"
                            max="10000"
                            step="0.01"
                            className="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                            required
                        />
                    </div>
                    <button
                        type="submit"
                        disabled={loading || fetching || wallets.length === 0}
                        className="w-full bg-blue-600 text-white p-2 rounded hover:bg-blue-700 disabled:bg-blue-300"
                    >
                        {loading ? 'Processing...' : 'Convert'}
                    </button>
                    {error && <p className="text-red-500 mt-4 text-center">{error}</p>}
                </form>
            )}
        </div>
    );
}

export default ConvertForm;