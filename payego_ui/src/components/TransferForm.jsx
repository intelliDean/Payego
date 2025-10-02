import React, { useState, useEffect } from 'react';
import axios from 'axios';
import { useNavigate } from 'react-router-dom';

function TransferForm() {
    const [amount, setAmount] = useState('');
    const [recipientEmail, setRecipientEmail] = useState('');
    const [bankCode, setBankCode] = useState('');
    const [accountNumber, setAccountNumber] = useState('');
    const [accountName, setAccountName] = useState('');
    const [currency, setCurrency] = useState('');
    const [transferType, setTransferType] = useState('internal');
    const [wallets, setWallets] = useState([]);
    const [banks, setBanks] = useState([]);
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(false);
    const [resolving, setResolving] = useState(false);
    const navigate = useNavigate();

    useEffect(() => {
        const fetchData = async () => {
            try {
                // Fetch user wallets
                const walletResponse = await axios.get(`${import.meta.env.VITE_API_URL}/api/wallets`, {
                    headers: { Authorization: `Bearer ${localStorage.getItem('jwt_token')}` },
                });
                setWallets(walletResponse.data.wallets || []);
                if (walletResponse.data.wallets.length > 0) {
                    setCurrency(walletResponse.data.wallets[0].currency);
                }

                // Fetch bank list from backend
                const bankResponse = await axios.get(`${import.meta.env.VITE_API_URL}/api/banks`);
                console.log("Banks:", bankResponse.data);
                setBanks(bankResponse.data.banks || []);
                console.log("Bank Code: ", bankResponse.data.banks[0].code);
                if (bankResponse.data.banks.length > 0) {

                    setBankCode(bankResponse.data.banks[0].code);
                }
            } catch (err) {
                setError(err.response?.data?.message || 'Failed to fetch wallets or banks');
            }
        };
        fetchData();
    }, []);

    useEffect(() => {
        setRecipientEmail('');
        setBankCode(banks.length > 0 ? banks[0].code : '');
        setAccountNumber('');
        setAccountName('');
        setError(null);
    }, [transferType, banks]);

    // Auto-resolve account when bankCode and accountNumber are valid
    useEffect(() => {
        let debounceTimer;
        const resolveAccount = async () => {
            if (!bankCode || !accountNumber) {
                setError(null);
                setAccountName('');
                return;
            }

            if (!/^\d{10}$/.test(accountNumber)) {
                setError(null); // Clear error until 10 digits are reached
                setAccountName('');
                return;
            }

            setResolving(true);
            setError(null);
            setAccountName('');

            try {
                console.log('Resolving account:', { bank_code: bankCode, account_number: accountNumber });
                const response = await axios.get(
                    `${import.meta.env.VITE_API_URL}/api/resolve_account`,
                    {
                        params: {
                            bank_code: bankCode.trim(),
                            account_number: accountNumber.trim(),
                        },
                    }
                );
                setAccountName(response.data.account_name);
            } catch (err) {
                console.error('Resolve account error:', err.response?.data || err.message);
                setError(err.response?.data?.message || 'Failed to resolve account. Please check bank details.');
            } finally {
                setResolving(false);
            }
        };

        // Debounce to avoid rapid API calls while typing
        if (transferType === 'external' && bankCode && accountNumber) {
            debounceTimer = setTimeout(resolveAccount, 500); // 500ms debounce
        }

        return () => clearTimeout(debounceTimer); // Cleanup timer
    }, [bankCode, accountNumber, transferType]);

    const handleSubmit = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);

        if (!amount || amount < 1 || amount > 10000) {
            setError('Amount must be between 1 and 10,000');
            setLoading(false);
            return;
        }

        if (!currency) {
            setError('Please select a currency');
            setLoading(false);
            return;
        }

        const selectedWallet = wallets.find(w => w.currency === currency);
        if (!selectedWallet) {
            setError(`No wallet found for ${currency}`);
            setLoading(false);
            return;
        }
        if (selectedWallet && (amount * 100) > selectedWallet.balance) {
            setError(`Insufficient balance: available ${(selectedWallet.balance / 100).toFixed(2)} ${currency}`);
            setLoading(false);
            return;
        }

        if (transferType === 'external') {
            if (!accountName) {
                setError('Please resolve the account name first');
                setLoading(false);
                return;
            }

            if (!/^\d{10}$/.test(accountNumber)) {
                setError('Account number must be 10 digits');
                setLoading(false);
                return;
            }
        } else if (transferType === 'internal' && !recipientEmail) {
            setError('Recipient email is required');
            setLoading(false);
            return;
        }

        try {
            const endpoint = transferType === 'internal' ? '/api/transfer/internal' : '/api/transfer/external';
            const payload = transferType === 'internal'
                ? { amount: parseFloat(amount), recipient_email: recipientEmail, currency }
                : { amount: parseFloat(amount), currency, bank_code: bankCode, account_number: accountNumber, account_name: accountName };

            const response = await axios.post(
                `${import.meta.env.VITE_API_URL}${endpoint}`,
                payload,
                {
                    headers: {
                        'Content-Type': 'application/json',
                        'Authorization': `Bearer ${localStorage.getItem('jwt_token')}`,
                    },
                }
            );

            if (transferType === 'external') {
                alert(`Transfer initiated! Transaction ID: ${response.data.transaction_id}`);
            } else {
                alert('Internal transfer initiated!');
            }
            navigate('/');
        } catch (err) {
            setError(err.response?.data?.message || `Failed to process ${transferType} transfer`);
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="max-w-md mx-auto mt-10 p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
            <div className="text-center mb-8">
                <div className="w-16 h-16 bg-gradient-to-r from-blue-500 to-indigo-500 rounded-2xl flex items-center justify-center mx-auto mb-4">
                    <span className="text-white text-2xl">💸</span>
                </div>
                <h2 className="text-3xl font-bold text-gray-800 mb-2">Transfer Funds</h2>
                <p className="text-gray-600">Send money to users or bank accounts</p>
            </div>
            
            <form onSubmit={handleSubmit}>
                <div className="mb-4">
                    <label className="block text-gray-700 font-medium mb-2">Transfer Type</label>
                    <select
                        value={transferType}
                        onChange={(e) => setTransferType(e.target.value)}
                        className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                    >
                        <option value="internal">Internal (Payego User)</option>
                        <option value="external">External (Bank Account)</option>
                    </select>
                </div>
                <div className="mb-4">
                    <label className="block text-gray-700 font-medium mb-2">Currency</label>
                    <select
                        value={currency}
                        onChange={(e) => setCurrency(e.target.value)}
                        className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                        required
                    >
                        <option value="" disabled>Select a currency</option>
                        {wallets.map((wallet) => (
                            <option key={wallet.currency} value={wallet.currency}>
                                {wallet.currency} (Balance: {(wallet.balance / 100).toFixed(2)})
                            </option>
                        ))}
                    </select>
                </div>
                <div className="mb-4">
                    <label className="block text-gray-700 font-medium mb-2">Amount ({currency || 'select currency'})</label>
                    <input
                        type="number"
                        value={amount}
                        onChange={(e) => setAmount(e.target.value)}
                        min="1"
                        max="10000"
                        step="0.01"
                        className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                        placeholder="Enter amount to transfer"
                        required
                    />
                </div>
                {transferType === 'internal' ? (
                    <div className="mb-4">
                        <label className="block text-gray-700 font-medium mb-2">Recipient Email</label>
                        <input
                            type="email"
                            value={recipientEmail}
                            onChange={(e) => setRecipientEmail(e.target.value)}
                            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                            placeholder="Enter recipient's email"
                            required
                        />
                    </div>
                ) : (
                    <>
                        <div className="mb-4">
                            <label className="block text-gray-700 font-medium mb-2">Bank</label>
                            <select
                                value={bankCode}
                                onChange={(e) => {
                                    setBankCode(e.target.value);
                                    setAccountName('');
                                }}
                                className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                                required
                            >
                                <option value="" disabled>Select a bank</option>
                                {banks.map((bank) => (
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
                                onChange={(e) => {
                                    setAccountNumber(e.target.value);
                                    setAccountName('');
                                }}
                                className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                                placeholder="Enter account number"
                                required
                            />
                            {accountNumber && !/^\d{10}$/.test(accountNumber) && (
                                <p className="text-red-500 text-sm mt-1">Account number must be 10 digits</p>
                            )}
                        </div>
                        <div className="mb-4">
                            <label className="block text-gray-700 font-medium mb-2">Account Name</label>
                            <div className="p-3 bg-gray-50 border border-gray-200 rounded-lg">
                                {resolving ? 'Resolving...' : accountName ? (
                                    <span>Account Name: <strong>{accountName}</strong></span>
                                ) : (
                                    'Enter a valid bank and account number to see account name'
                                )}
                            </div>
                        </div>
                    </>
                )}
                <button
                    type="submit"
                    disabled={loading || (transferType === 'external' && (banks.length === 0 || !accountName))}
                    className="w-full bg-gradient-to-r from-blue-500 to-indigo-500 text-white p-3 rounded-lg hover:from-blue-600 hover:to-indigo-600 disabled:from-gray-400 disabled:to-gray-400 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                >
                    {loading ? 'Processing...' : 'Transfer'}
                </button>
                {error && (
                    <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                        <p className="text-red-600 text-center text-sm">{error}</p>
                    </div>
                )}
            </form>
        </div>
    );
}

export default TransferForm;