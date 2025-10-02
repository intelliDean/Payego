import React, { useState, useEffect } from "react";
import axios from "axios";
import { Link, useNavigate } from "react-router-dom";
import ErrorBoundary from "./ErrorBoundary";

const SUPPORTED_CURRENCIES = [
    'USD', 'EUR', 'GBP', 'AUD', 'BRL', 'CAD', 'CHF', 'CNY', 'HKD', 'INR',
    'JPY', 'KRW', 'MXN', 'NGN', 'NOK', 'NZD', 'SEK', 'SGD', 'TRY', 'ZAR',
].sort((a, b) => a.localeCompare(b, 'en', { sensitivity: 'base' }));

function ConvertForm() {
    const [amount, setAmount] = useState("");
    const [fromCurrency, setFromCurrency] = useState("");
    const [toCurrency, setToCurrency] = useState("");
    const [wallets, setWallets] = useState([]);
    const [supportedCurrencies, setSupportedCurrencies] = useState([]);
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(false);
    const [fetching, setFetching] = useState(true);
    const navigate = useNavigate();

    // useEffect(() => {
    //     const fetchData = async () => {
    //         try {
    //             setFetching(true);
    //             const token = localStorage.getItem("jwt_token") || sessionStorage.getItem("jwt_token");
    //             if (!token) {
    //                 setError("No session found. Time to join the Payego party!");
    //                 navigate("/login");
    //                 return;
    //             }
    //
    //             const [walletsResponse, currenciesResponse] = await Promise.all([
    //                 axios.get(`${import.meta.env.VITE_API_URL}/api/wallets`, {
    //                     headers: { Authorization: `Bearer ${token}` },
    //                 }),
    //                 axios.get(`${import.meta.env.VITE_API_URL}/api/currencies`, {
    //                     headers: { Authorization: `Bearer ${token}` },
    //                 }),
    //             ]);
    //
    //             console.log("Fetched wallets:", walletsResponse.data.wallets); // Debug
    //             setWallets(walletsResponse.data.wallets || []);
    //             if (walletsResponse.data.wallets?.length > 0) {
    //                 setFromCurrency(walletsResponse.data.wallets[0].currency);
    //             }
    //
    //             console.log("Fetched currencies:", currenciesResponse.data.currencies); // Debug
    //             setSupportedCurrencies(currenciesResponse.data.currencies || []);
    //             if (currenciesResponse.data.currencies?.length > 0) {
    //                 setToCurrency(currenciesResponse.data.currencies[0]);
    //             }
    //         } catch (err) {
    //             if (err.response?.status === 401) {
    //                 setError("Session expired. Back to the login gate!");
    //                 localStorage.removeItem("jwt_token");
    //                 sessionStorage.removeItem("jwt_token");
    //                 navigate("/login");
    //             } else {
    //                 setError(err.response?.data?.message || "Wallets ran off to Vegas! Try again!");
    //             }
    //         } finally {
    //             setFetching(false);
    //         }
    //     };
    //     fetchData();
    // }, [navigate]);

    useEffect(() => {
        const fetchData = async () => {
            try {
                setFetching(true);
                const token = localStorage.getItem('jwt_token') || sessionStorage.getItem('jwt_token');
                if (!token) {
                    setError('No session found. Time to join the Payego party!');
                    navigate('/login');
                    return;
                }

                const walletsResponse = await axios.get(`${import.meta.env.VITE_API_URL}/api/wallets`, {
                    headers: { Authorization: `Bearer ${token}` },
                });

                console.log('Fetched wallets:', walletsResponse.data.wallets); // Debug
                setWallets(walletsResponse.data.wallets || []);
                if (walletsResponse.data.wallets?.length > 0) {
                    setFromCurrency(walletsResponse.data.wallets[0].currency);
                }

                console.log('Hardcoded currencies:', SUPPORTED_CURRENCIES); // Debug
                setSupportedCurrencies(SUPPORTED_CURRENCIES);
                if (SUPPORTED_CURRENCIES.length > 0) {
                    setToCurrency(SUPPORTED_CURRENCIES[0]);
                }
            } catch (err) {
                if (err.response?.status === 401) {
                    setError('Session expired. Back to the login gate!');
                    localStorage.removeItem('jwt_token');
                    sessionStorage.removeItem('jwt_token');
                    navigate('/login');
                } else {
                    setError(err.response?.data?.message || 'Wallets ran off to Vegas! Try again!');
                }
            } finally {
                setFetching(false);
            }
        };
        fetchData();
    }, [navigate]);

    const handleSubmit = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);

        if (!amount || parseFloat(amount) < 0.01 || parseFloat(amount) > 10000) {
            setError("Amount must be between 0.01 and 10,000. Don't get too wild!");
            setLoading(false);
            return;
        }
        if (fromCurrency === toCurrency) {
            setError("From and to currencies can't be twins!");
            setLoading(false);
            return;
        }
        const selectedWallet = wallets.find(w => w.currency === fromCurrency);
        if (!selectedWallet) {
            setError(`No wallet found for ${fromCurrency}. Where's it hiding?`);
setLoading(false);
return;
}
if (parseFloat(amount) * 100 > selectedWallet.balance) {
    setError(`Not enough coins! Available: ${(selectedWallet.balance / 100).toFixed(2)} ${fromCurrency}`);
    setLoading(false);
    return;
}

try {
    const token = localStorage.getItem("jwt_token") || sessionStorage.getItem("jwt_token");
    const response = await axios.post(
        `${import.meta.env.VITE_API_URL}/api/convert_currency`,
        {
            amount: parseFloat(amount),
            from_currency: fromCurrency,
            to_currency: toCurrency,
        },
        {
            headers: {
                "Content-Type": "application/json",
                Authorization: `Bearer ${token}`,
            },
        }
    );
    alert(`Conversion completed! Transaction ID: ${response.data.transaction_id}, Converted: ${response.data.converted_amount} ${toCurrency}`);
    navigate("/");
} catch (err) {
    if (err.response?.status === 401) {
        setError("Session expired. Back to the login gate!");
        localStorage.removeItem("jwt_token");
        sessionStorage.removeItem("jwt_token");
        navigate("/login");
    } else {
        setError(err.response?.data?.message || "Conversion took a wrong turn!");
    }
} finally {
    setLoading(false);
}
};

const handleCancel = () => {
    navigate("/");
};

return (
    <ErrorBoundary>
        <div className="max-w-md mx-auto mt-10 p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
            <div className="text-center mb-8">
                <div className="w-16 h-16 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-2xl flex items-center justify-center mx-auto mb-4">
                    <span className="text-white text-2xl">🔄</span>
                </div>
                <h2 className="text-3xl font-bold text-gray-800 mb-2">Convert Currency</h2>
                <p className="text-gray-600">Swap your coins with ease</p>
            </div>

            {fetching ? (
                <div className="flex flex-col items-center justify-center py-12">
                    <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                    <p className="mt-4 text-gray-600">Fetching your wallets...</p>
                </div>
            ) : wallets.length === 0 ? (
                <div className="text-center py-12">
                    <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
                        <span className="text-gray-400 text-2xl">💸</span>
                    </div>
                    <p className="text-gray-600 mb-4">No wallets yet! Add one to start swapping!</p>
                </div>
            ) : (
                <form onSubmit={handleSubmit}>
                    <div className="mb-4">
                        <label htmlFor="from-currency" className="block text-gray-700 font-medium mb-2">
                            From Currency
                        </label>
                        <select
                            id="from-currency"
                            value={fromCurrency}
                            onChange={(e) => setFromCurrency(e.target.value)}
                            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                            required
                            aria-describedby={error && !fromCurrency ? "from-currency-error" : undefined}
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
                        <label htmlFor="to-currency" className="block text-gray-700 font-medium mb-2">
                            To Currency
                        </label>
                        <select
                            id="to-currency"
                            value={toCurrency}
                            onChange={(e) => setToCurrency(e.target.value)}
                            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                            required
                            aria-describedby={error && !toCurrency ? "to-currency-error" : undefined}
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
                        <label htmlFor="amount" className="block text-gray-700 font-medium mb-2">
                            Amount
                        </label>
                        <input
                            id="amount"
                            type="number"
                            value={amount}
                            onChange={(e) => {
                                const value = e.target.value;
                                if (value === "" || (/^\d*\.?\d{0,2}$/.test(value) && parseFloat(value) >= 0)) {
                                    setAmount(value);
                                }
                            }}
                            min="0.01"
                            max="10000"
                            step="0.01"
                            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                            placeholder="Enter amount to convert (0.01 - 10,000)"
                            required
                            aria-describedby={error && amount ? "amount-error" : undefined}
                        />
                    </div>
                    <div className="flex space-x-4">
                        <button
                            type="submit"
                            disabled={loading || fetching || wallets.length === 0 || supportedCurrencies.length === 0}
                            className="flex-1 bg-gradient-to-r from-blue-600 to-indigo-600 text-white p-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 disabled:from-gray-400 disabled:to-gray-400 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                            aria-label="Convert currency"
                        >
                            {loading ? "Processing..." : "Convert"}
                        </button>
                        <Link
                            to="/"
                            onClick={handleCancel}
                            className="flex-1 bg-gray-200 text-gray-700 p-3 rounded-lg hover:bg-gray-300 transition-all duration-200 font-medium text-center"
                            aria-label="Cancel conversion"
                        >
                            Cancel
                        </Link>
                    </div>
                    {error && (
                        <div id="error-message" className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                            <p className="text-red-600 text-center text-sm">{error}</p>
                        </div>
                    )}
                </form>
            )}
        </div>
    </ErrorBoundary>
);
}

export default ConvertForm;
