import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { useNavigate } from 'react-router-dom';

function WithdrawForm() {
    const [amount, setAmount] = useState('');
    const [currency, setCurrency] = useState('');
    const [bankAccountId, setBankAccountId] = useState('');
    const [wallets, setWallets] = useState([]);
    const [bankAccounts, setBankAccounts] = useState([]);
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(false);
    const navigate = useNavigate();

    useEffect(() => {
        const fetchData = async () => {
            try {
                const [walletsResponse, bankAccountsResponse] = await Promise.all([
                    axios.get(`${import.meta.env.VITE_API_URL}/api/wallets`, {
                        headers: { 'Authorization': `Bearer ${localStorage.getItem('jwt_token')}` },
                    }),
                    axios.get(`${import.meta.env.VITE_API_URL}/api/bank_accounts`, {
                        headers: { 'Authorization': `Bearer ${localStorage.getItem('jwt_token')}` },
                    }),
                ]);
                console.log("Wallet: ", walletsResponse.data.wallets);
                console.log("Banks: ", bankAccountsResponse.data.bank_accounts);
                setWallets(walletsResponse.data.wallets);
                setBankAccounts(bankAccountsResponse.data.bank_accounts);
                if (walletsResponse.data.wallets.length > 0) {
                    setCurrency(walletsResponse.data.wallets[0].currency);
                }
                if (bankAccountsResponse.data.bank_accounts.length > 0) {
                    setBankAccountId(bankAccountsResponse.data.bank_accounts[0].bank_id);
                }
            } catch (err) {
                setError(err.response?.data?.message || 'Failed to fetch data');
            }
        };
        fetchData();
    }, []);

    const handleSubmit = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);

        if (bankAccounts.length === 0) {
            setError('No bank accounts available. Please add a bank account.');
            setLoading(false);
            navigate('/banks/add');
            return;
        }

        const selectedWallet = wallets.find(w => w.currency === currency);
        const maxAmount = currency === 'NGN' ? 10000000 : 10000; // 10M NGN, 10K others
        if (!amount || amount < 1 || amount > maxAmount) {
            setError(`Amount must be between 1 and ${maxAmount} ${currency}`);
            setLoading(false);
            return;
        }
        if (!currency) {
            setError('Please select a currency');
            setLoading(false);
            return;
        }
        if (!bankAccountId) {
            setError('Please select a bank account');
            setLoading(false);
            return;
        }
        if (selectedWallet && (amount * 100) > selectedWallet.balance) {
            setError(`Insufficient balance: available ${(selectedWallet.balance / 100).toFixed(2)} ${currency}`);
            setLoading(false);
            return;
        }

        try {
            const response = await axios.post(
                `${import.meta.env.VITE_API_URL}/api/withdraw`,
                {
                    amount: parseFloat(amount),
                    currency,
                    bank_id: bankAccountId,
                },
                {
                    headers: {
                        'Content-Type': 'application/json',
                        'Authorization': `Bearer ${localStorage.getItem('jwt_token')}`,
                    },
                }
            );
            console.log(`Withdrawal initiated: transaction_id=${response.data.transaction_id}`);
            navigate('/');
        } catch (err) {
            setError(err.response?.data?.message || 'Failed to process withdrawal');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="max-w-md mx-auto mt-10 p-6 bg-white rounded-lg shadow-md">
            <h2 className="text-2xl font-bold mb-6 text-center">Withdraw Funds</h2>
            <form onSubmit={handleSubmit}>
                <div className="mb-4">
                    <label className="block text-gray-700">Currency</label>
                    <select
                        value={currency}
                        onChange={(e) => setCurrency(e.target.value)}
                        className="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                        required
                    >
                        <option value="" disabled>Select currency</option>
                        {wallets.map(wallet => (
                            <option key={wallet.currency} value={wallet.currency}>
                                {wallet.currency} (Balance: {(wallet.balance / 100).toFixed(2)})
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
                        max={currency === 'NGN' ? '10000000' : '10000'}
                        step="0.01"
                        className="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                        required
                    />
                </div>
                <div className="mb-4">
                    <label className="block text-gray-700">Bank Account</label>
                    <select
                        value={bankAccountId}
                        onChange={(e) => setBankAccountId(e.target.value)}
                        className="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                        required
                    >
                        <option value="" disabled>Select bank account</option>
                        {bankAccounts.map(account => (
                            <option key={account.id} value={account.id}>
                                {account.account_name || 'Bank Account'} - {account.account_number} ({account.bank_name || account.bank_code})
                            </option>
                        ))}
                    </select>
                </div>
                <button
                    type="submit"
                    disabled={loading || wallets.length === 0 || bankAccounts.length === 0}
                    className="w-full bg-blue-600 text-white p-2 rounded hover:bg-blue-700 disabled:bg-blue-300"
                >
                    {loading ? 'Processing...' : 'Withdraw'}
                </button>
                {error && <p className="text-red-500 mt-4 text-center">{error}</p>}
            </form>
        </div>
    );
}

export default WithdrawForm;