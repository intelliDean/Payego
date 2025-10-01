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
        <div className="max-w-md mx-auto mt-10 p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
            <div className="text-center mb-8">
                <div className="w-16 h-16 bg-gradient-to-r from-orange-500 to-red-500 rounded-2xl flex items-center justify-center mx-auto mb-4">
                    <span className="text-white text-2xl">🔄</span>
                </div>
                <h2 className="text-3xl font-bold text-gray-800 mb-2">Convert Currency</h2>
                <p className="text-gray-600">Exchange between different currencies</p>
            </div>
            
            {fetching ? (
                <div className="flex justify-center items-center py-12">
                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-orange-600"></div>
                </div>
            ) : (
                <form onSubmit={handleSubmit}>
                    <div className="mb-4">
                        <label className="block text-gray-700 font-medium mb-2">From Currency</label>
                        <select
                            value={fromCurrency}
                            onChange={(e) => setFromCurrency(e.target.value)}
                            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-orange-500 focus:border-transparent transition-all duration-200"
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
                        <label className="block text-gray-700 font-medium mb-2">To Currency</label>
                        <select
                            value={toCurrency}
                            onChange={(e) => setToCurrency(e.target.value)}
                            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-orange-500 focus:border-transparent transition-all duration-200"
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
                        <label className="block text-gray-700 font-medium mb-2">Amount</label>
                        <input
                            type="number"
                            value={amount}
                            onChange={(e) => setAmount(e.target.value)}
                            min="1"
                            max="10000"
                            step="0.01"
                            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-orange-500 focus:border-transparent transition-all duration-200"
                            placeholder="Enter amount to convert"
                            required
                        />
                    </div>
                    <button
                        type="submit"
                        disabled={loading || fetching || wallets.length === 0}
                        className="w-full bg-gradient-to-r from-orange-500 to-red-500 text-white p-3 rounded-lg hover:from-orange-600 hover:to-red-600 disabled:from-gray-400 disabled:to-gray-400 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                    >
                        {loading ? 'Processing...' : 'Convert'}
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

export default ConvertForm;