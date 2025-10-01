// import React, { useState, useEffect } from 'react';
// import axios from 'axios';
// import { Link } from 'react-router-dom';
//
// function Dashboard() {
//     const [user, setUser] = useState(null);
//     const [error, setError] = useState(null);
//     const [loading, setLoading] = useState(true);
//
//     useEffect(() => {
//         const fetchUser = async () => {
//             try {
//                 const response = await axios.get(`${import.meta.env.VITE_API_URL}/api/current_user`, {
//                     headers: {
//                         'Authorization': `Bearer ${localStorage.getItem('jwt_token')}`,
//                     },
//                 });
//                 setUser(response.data);
//             } catch (err) {
//                 setError(err.response?.data?.message || 'Failed to fetch user data');
//             } finally {
//                 setLoading(false);
//             }
//         };
//         fetchUser();
//     }, []);
//
//     // Format balance based on currency
//     const formatBalance = (balance, currency) => {
//         return new Intl.NumberFormat('en-US', {
//             style: 'currency',
//             currency: currency,
//             minimumFractionDigits: 2,
//         }).format(balance / 100);
//     };
//
//     return (
//         <div className="max-w-4xl mx-auto">
//             <div className="mb-8">
//                 <h1 className="text-4xl font-bold text-gray-800 mb-2">Dashboard</h1>
//                 <p className="text-gray-600">Manage your finances with ease</p>
//             </div>
//
//             {loading && (
//                 <div className="flex justify-center items-center py-12">
//                     <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
//                 </div>
//             )}
//
//             {error && (
//                 <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
//                     <p className="text-red-600 text-center">{error}</p>
//                 </div>
//             )}
//
//             {user && (
//                 <div className="space-y-8">
//                     {/* User Info Card */}
//                     <div className="bg-white rounded-2xl shadow-lg p-6 border border-gray-100">
//                         <div className="flex items-center space-x-4 mb-6">
//                             <div className="w-12 h-12 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-full flex items-center justify-center">
//                                 <span className="text-white font-bold text-lg">{user.email.charAt(0).toUpperCase()}</span>
//                             </div>
//                             <div>
//                                 <h2 className="text-xl font-semibold text-gray-800">Welcome back!</h2>
//                                 <p className="text-gray-600">{user.email}</p>
//                             </div>
//                         </div>
//                     </div>
//
//                     {/* Wallet Balances */}
//                     <div className="bg-white rounded-2xl shadow-lg p-6 border border-gray-100">
//                         <h3 className="text-2xl font-bold text-gray-800 mb-6">Wallet Balances</h3>
//                         {user.wallets && user.wallets.length > 0 ? (
//                             <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
//                                 {user.wallets.map((wallet) => (
//                                     <div key={wallet.currency} className="bg-gradient-to-r from-blue-50 to-indigo-50 rounded-xl p-4 border border-blue-100">
//                                         <div className="flex items-center justify-between">
//                                             <div>
//                                                 <p className="text-sm font-medium text-gray-600">{wallet.currency}</p>
//                                                 <p className="text-2xl font-bold text-gray-800">
//                                                     {formatBalance(wallet.balance, wallet.currency)}
//                                                 </p>
//                                             </div>
//                                             <div className="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
//                                                 <span className="text-blue-600 font-bold text-sm">{wallet.currency}</span>
//                                             </div>
//                                         </div>
//                                     </div>
//                                 ))}
//                             </div>
//                         ) : (
//                             <div className="text-center py-8">
//                                 <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
//                                     <span className="text-gray-400 text-2xl">💳</span>
//                                 </div>
//                                 <p className="text-gray-600">No wallets found. Start by adding funds to your account.</p>
//                             </div>
//                         )}
//                     </div>
//
//                     {/* Quick Actions */}
//                     <div className="bg-white rounded-2xl shadow-lg p-6 border border-gray-100">
//                         <h3 className="text-2xl font-bold text-gray-800 mb-6">Quick Actions</h3>
//                         <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">
//                             <Link
//                                 to="/top-up"
//                                 className="group bg-gradient-to-r from-green-500 to-emerald-500 text-white p-6 rounded-xl hover:from-green-600 hover:to-emerald-600 transition-all duration-200 shadow-lg hover:shadow-xl transform hover:-translate-y-1"
//                             >
//                                 <div className="flex items-center space-x-3">
//                                     <span className="text-2xl">💰</span>
//                                     <div>
//                                         <h4 className="font-semibold">Top Up</h4>
//                                         <p className="text-sm opacity-90">Add funds to your wallet</p>
//                                     </div>
//                                 </div>
//                             </Link>
//
//                             <Link
//                                 to="/transfer"
//                                 className="group bg-gradient-to-r from-blue-500 to-indigo-500 text-white p-6 rounded-xl hover:from-blue-600 hover:to-indigo-600 transition-all duration-200 shadow-lg hover:shadow-xl transform hover:-translate-y-1"
//                             >
//                                 <div className="flex items-center space-x-3">
//                                     <span className="text-2xl">💸</span>
//                                     <div>
//                                         <h4 className="font-semibold">Transfer</h4>
//                                         <p className="text-sm opacity-90">Send money to others</p>
//                                     </div>
//                                 </div>
//                             </Link>
//
//                             <Link
//                                 to="/withdraw"
//                                 className="group bg-gradient-to-r from-purple-500 to-pink-500 text-white p-6 rounded-xl hover:from-purple-600 hover:to-pink-600 transition-all duration-200 shadow-lg hover:shadow-xl transform hover:-translate-y-1"
//                             >
//                                 <div className="flex items-center space-x-3">
//                                     <span className="text-2xl">🏦</span>
//                                     <div>
//                                         <h4 className="font-semibold">Withdraw</h4>
//                                         <p className="text-sm opacity-90">Transfer to bank account</p>
//                                     </div>
//                                 </div>
//                             </Link>
//
//                             <Link
//                                 to="/convert"
//                                 className="group bg-gradient-to-r from-orange-500 to-red-500 text-white p-6 rounded-xl hover:from-orange-600 hover:to-red-600 transition-all duration-200 shadow-lg hover:shadow-xl transform hover:-translate-y-1"
//                             >
//                                 <div className="flex items-center space-x-3">
//                                     <span className="text-2xl">🔄</span>
//                                     <div>
//                                         <h4 className="font-semibold">Convert</h4>
//                                         <p className="text-sm opacity-90">Exchange currencies</p>
//                                     </div>
//                                 </div>
//                             </Link>
//
//                             <Link
//                                 to="/add-bank"
//                                 className="group bg-gradient-to-r from-teal-500 to-cyan-500 text-white p-6 rounded-xl hover:from-teal-600 hover:to-cyan-600 transition-all duration-200 shadow-lg hover:shadow-xl transform hover:-translate-y-1"
//                             >
//                                 <div className="flex items-center space-x-3">
//                                     <span className="text-2xl">🏛️</span>
//                                     <div>
//                                         <h4 className="font-semibold">Manage Banks</h4>
//                                         <p className="text-sm opacity-90">Add bank accounts</p>
//                                     </div>
//                                 </div>
//                             </Link>
//                         </div>
//                     </div>
//                 </div>
//             )}
//         </div>
//     );
// }
//
// export default Dashboard;



import React, { useState, useEffect } from "react";
import axios from "axios";
import { Link, useNavigate } from "react-router-dom";
import ErrorBoundary from "./ErrorBoundary";

function Dashboard() {
    const [user, setUser] = useState(null);
    const [transactions, setTransactions] = useState([]);
    const [selectedTransaction, setSelectedTransaction] = useState(null);
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(true);
    const [sidebarOpen, setSidebarOpen] = useState(localStorage.getItem("sidebarOpen") !== "false");
    const navigate = useNavigate();

    useEffect(() => {
        const fetchData = async () => {
            try {
                const token = localStorage.getItem("jwt_token") || sessionStorage.getItem("jwt_token");
                if (!token) {
                    setError("No session found. Time to join the Payego party!");
                    navigate("/login");
                    return;
                }

                const [userResponse, transactionsResponse] = await Promise.all([
                    axios.get(`${import.meta.env.VITE_API_URL}/api/current_user`, {
                        headers: { Authorization: `Bearer ${token}` },
                    }),
                    axios.get(`${import.meta.env.VITE_API_URL}/api/transactions`, {
                        headers: { Authorization: `Bearer ${token}` },
                    }),
                ]);

                console.log("Fetched user:", userResponse.data);
                console.log("Fetched transactions:", transactionsResponse.data.transactions);
                setUser(userResponse.data || {});
                setTransactions(transactionsResponse.data.transactions?.slice(0, 3) || []);
            } catch (err) {
                if (err.response?.status === 401) {
                    setError("Session expired. Back to the login gate!");
                    localStorage.removeItem("jwt_token");
                    sessionStorage.removeItem("jwt_token");
                    navigate("/login");
                } else {
                    setError(err.response?.data?.message || "Data ran off to Vegas! Try again!");
                }
            } finally {
                setLoading(false);
            }
        };
        fetchData();
    }, [navigate]);

    const fetchTransactionDetails = async (id) => {
        try {
            const token = localStorage.getItem("jwt_token") || sessionStorage.getItem("jwt_token");
            const response = await axios.get(`${import.meta.env.VITE_API_URL}/api/transactions/${id}`, {
                headers: { Authorization: `Bearer ${token}` },
            });
            setSelectedTransaction(response.data);
        } catch (err) {
            setError(err.response?.data?.message || "Transaction details got lost in the void!");
        }
    };

    const handleLogout = () => {
        localStorage.removeItem("jwt_token");
        sessionStorage.removeItem("jwt_token");
        navigate("/login");
    };

    const toggleSidebar = () => {
        const newState = !sidebarOpen;
        setSidebarOpen(newState);
        localStorage.setItem("sidebarOpen", newState);
    };

    const formatBalance = (balance, currency) => {
        return new Intl.NumberFormat("en-US", {
            style: "currency",
            currency: currency || "USD",
            minimumFractionDigits: 2,
        }).format((balance || 0) / 100);
    };

    const formatDate = (dateStr) => {
        return new Date(dateStr).toLocaleDateString("en-US", {
            month: "short",
            day: "numeric",
            year: "numeric",
            hour: "2-digit",
            minute: "2-digit",
        });
    };

    // Group balances by currency
    const balancesByCurrency = user?.wallets?.reduce((acc, wallet) => {
        if (wallet.currency) {
            acc[wallet.currency] = (acc[wallet.currency] || 0) + (wallet.balance || 0);
        }
        return acc;
    }, {}) || {};

    return (
        <ErrorBoundary>
            <div className="min-h-screen bg-gray-50">
                {/* Sidebar */}
                <div className={`fixed inset-y-0 left-0 w-64 bg-white shadow-xl transform ${sidebarOpen ? "translate-x-0" : "-translate-x-full"} transition-transform duration-300 ease-in-out z-30`}>
                    <div className="p-6">
                        <h2 className="text-2xl font-bold text-gray-800 mb-8">Payego</h2>
                        <nav className="space-y-2">
                            {[
                                { to: "/dashboard", label: "Dashboard", icon: "🏠" },
                                { to: "/top-up", label: "Top Up", icon: "💰" },
                                { to: "/transfer", label: "Transfer", icon: "💸" },
                                { to: "/withdraw", label: "Withdraw", icon: "🏦" },
                                { to: "/convert", label: "Convert", icon: "🔄" },
                                { to: "/banks", label: "Banks", icon: "🏛️" },
                                { to: "/wallets", label: "Wallets", icon: "💳" },
                                { to: "/transactions", label: "Transactions", icon: "📜" },
                                { to: "/profile", label: "Profile", icon: "👤" },
                            ].map((item) => (
                                <Link
                                    key={item.to}
                                    to={item.to}
                                    className="flex items-center space-x-3 p-3 rounded-lg hover:bg-gradient-to-r from-blue-50 to-indigo-50 text-gray-700 hover:text-blue-700 transition-all duration-200"
                                    onClick={() => setSidebarOpen(false)}
                                    aria-label={item.label}
                                >
                                    <span className="text-lg">{item.icon}</span>
                                    <span className="font-medium">{item.label}</span>
                                </Link>
                            ))}
                            <button
                                onClick={handleLogout}
                                className="flex items-center space-x-3 p-3 rounded-lg hover:bg-red-50 text-gray-700 hover:text-red-700 transition-all duration-200 w-full text-left"
                                aria-label="Log out"
                            >
                                <span className="text-lg">🚪</span>
                                <span className="font-medium">Log Out</span>
                            </button>
                        </nav>
                    </div>
                </div>

                {/* Main Content */}
                <div className={`p-4 md:p-6 transition-all duration-300 ${sidebarOpen ? "md:ml-64" : "md:ml-0"}`}>
                    <div className="max-w-5xl mx-auto">
                        <div className="flex justify-between items-center mb-6">
                            <div className="flex items-center space-x-3">
                                <button
                                    className="p-2 bg-gradient-to-r from-blue-600 to-indigo-600 text-white rounded-lg"
                                    onClick={toggleSidebar}
                                    aria-label={sidebarOpen ? "Collapse sidebar" : "Expand sidebar"}
                                >
                                    <span className="text-lg">{sidebarOpen ? "◄" : "☰"}</span>
                                </button>
                                <div>
                                    <h1 className="text-2xl font-bold text-gray-900">Dashboard</h1>
                                    <p className="text-sm text-gray-500">Your financial snapshot</p>
                                </div>
                            </div>
                            {user && (
                                <Link
                                    to="/profile"
                                    className="flex items-center space-x-2 bg-white rounded-lg px-3 py-2 shadow-sm hover:bg-gray-100 transition-all duration-200"
                                    aria-label="View profile"
                                >
                                    <div className="w-8 h-8 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-full flex items-center justify-center">
                                        <span className="text-white font-bold">
                                            {user.email?.charAt(0)?.toUpperCase() || "?"}
                                        </span>
                                    </div>
                                    <p className="text-sm font-medium text-gray-900 hidden sm:block">{user.email || "User"}</p>
                                </Link>
                            )}
                        </div>

                        {loading && (
                            <div className="flex flex-col items-center justify-center h-64">
                                <div className="animate-spin rounded-full h-10 w-10 border-b-2 border-blue-600"></div>
                                <p className="mt-3 text-gray-600">Fetching your dashboard...</p>
                            </div>
                        )}

                        {error && (
                            <div className="rounded-lg bg-red-50 p-3 mb-4">
                                <div className="flex">
                                    <div className="flex-shrink-0">
                                        <svg className="h-5 w-5 text-red-400" viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
                                            <path
                                                fillRule="evenodd"
                                                d="M10 18a8 8 0 100-16 8 8 0 000 16zM8.707 7.293a1 1 0 00-1.414 1.414L8.586 10l-1.293 1.293a1 1 0 101.414 1.414L10 11.414l1.293 1.293a1 1 0 001.414-1.414L11.414 10l1.293-1.293a1 1 0 00-1.414-1.414L10 8.586 8.707 7.293z"
                                                clipRule="evenodd"
                                            />
                                        </svg>
                                    </div>
                                    <div className="ml-3">
                                        <p className="text-sm font-medium text-red-800">{error}</p>
                                    </div>
                                </div>
                            </div>
                        )}

                        {user && (
                            <div className="space-y-4">
                                {/* Stats */}
                                <div className="bg-white rounded-lg shadow-sm border border-gray-100">
                                    <div className="p-4">
                                        <h3 className="text-lg font-semibold text-gray-900 mb-3">Balances</h3>
                                        {Object.keys(balancesByCurrency).length > 0 ? (
                                            <div className="grid grid-cols-1 sm:grid-cols-2 gap-3">
                                                {Object.entries(balancesByCurrency).map(([currency, balance]) => (
                                                    <div
                                                        key={currency}
                                                        className="bg-gradient-to-r from-blue-50 to-indigo-50 rounded-lg p-3 border border-blue-100"
                                                        role="region"
                                                        aria-label={`Balance: ${currency}`}
                                                    >
                                                        <h4 className="text-sm font-medium text-gray-500">{currency}</h4>
                                                        <p className="text-lg font-bold text-gray-900">
                                                            {formatBalance(balance, currency)}
                                                        </p>
                                                    </div>
                                                ))}
                                            </div>
                                        ) : (
                                            <div className="text-center py-4 bg-gray-50 rounded-b-lg">
                                                <div className="inline-flex items-center justify-center w-10 h-10 bg-gray-100 rounded-full mb-2">
                                                    <span className="text-lg">💳</span>
                                                </div>
                                                <p className="text-gray-600 text-sm">No wallets yet! Add funds to join the Payego party!</p>
                                            </div>
                                        )}
                                    </div>
                                </div>

                                {/* Wallets */}
                                <div className="bg-white rounded-lg shadow-sm border border-gray-100">
                                    <div className="p-4 flex justify-between items-center">
                                        <h2 className="text-lg font-semibold text-gray-900">Wallets</h2>
                                        <Link to="/wallets" className="text-sm text-blue-600 hover:text-blue-700" aria-label="View all wallets">
                                            View All
                                        </Link>
                                    </div>
                                    {user.wallets?.length > 0 ? (
                                        <div className="grid grid-cols-1 gap-3 p-4 pt-0">
                                            {user.wallets.slice(0, 3).map((wallet) => (
                                                <div
                                                    key={wallet.currency}
                                                    className="bg-gradient-to-r from-blue-50 to-indigo-50 rounded-lg p-3 border border-blue-100 hover:from-blue-100 hover:to-indigo-100 transition-all duration-200"
                                                    role="region"
                                                    aria-label={`Wallet: ${wallet.currency}`}
                                                >
                                                    <div className="flex items-center justify-between">
                                                        <div className="flex items-center space-x-2">
                                                            <div className="w-8 h-8 bg-blue-100 rounded-lg flex items-center justify-center">
                                                                <span className="text-blue-600 font-medium text-sm">
                                                                    {wallet.currency || "N/A"}
                                                                </span>
                                                            </div>
                                                            <span className="text-sm font-medium text-gray-500">Balance</span>
                                                        </div>
                                                        <p className="text-base font-bold text-gray-900">
                                                            {formatBalance(wallet.balance, wallet.currency)}
                                                        </p>
                                                    </div>
                                                </div>
                                            ))}
                                        </div>
                                    ) : (
                                        <div className="text-center py-4 bg-gray-50 rounded-b-lg">
                                            <div className="inline-flex items-center justify-center w-10 h-10 bg-gray-100 rounded-full mb-2">
                                                <span className="text-lg">💳</span>
                                            </div>
                                            <p className="text-gray-600 text-sm">No wallets yet! Add funds to join the Payego party!</p>
                                        </div>
                                    )}
                                </div>

                                {/* Transactions */}
                                <div className="bg-white rounded-lg shadow-sm border border-gray-100">
                                    <div className="p-4 flex justify-between items-center">
                                        <h2 className="text-lg font-semibold text-gray-900">Recent Transactions</h2>
                                        <Link to="/transactions" className="text-sm text-blue-600 hover:text-blue-700" aria-label="View all transactions">
                                            View All
                                        </Link>
                                    </div>
                                    {transactions.length > 0 ? (
                                        <div className="divide-y divide-gray-200">
                                            {transactions.map((tx) => (
                                                <button
                                                    key={tx.id}
                                                    onClick={() => fetchTransactionDetails(tx.id)}
                                                    className="w-full flex items-center justify-between p-3 hover:bg-gray-50 transition-all duration-200"
                                                    aria-label={`View details for ${tx.type} transaction`}
                                                >
                                                    <div className="flex items-center space-x-2">
                                                        <div className="w-6 h-6 bg-gray-100 rounded-lg flex items-center justify-center">
                                                            <span className="text-sm">
                                                                {tx.type === "top_up" ? "💰" : tx.type === "transfer" ? "💸" : tx.type === "withdraw" ? "🏦" : "🔄"}
                                                            </span>
                                                        </div>
                                                        <div className="text-left">
                                                            <p className="text-sm font-medium text-gray-900 capitalize">{tx.type.replace("_", " ")}</p>
                                                            <p className="text-xs text-gray-500">{formatDate(tx.created_at)}</p>
                                                        </div>
                                                    </div>
                                                    <div className="text-right">
                                                        <p className={`text-sm font-medium ${tx.amount >= 0 ? "text-green-600" : "text-red-600"}`}>
                                                            {formatBalance(tx.amount, tx.currency)}
                                                        </p>
                                                        <p className="text-xs text-gray-500 capitalize">{tx.status}</p>
                                                    </div>
                                                </button>
                                            ))}
                                        </div>
                                    ) : (
                                        <div className="text-center py-4 bg-gray-50 rounded-b-lg">
                                            <div className="inline-flex items-center justify-center w-10 h-10 bg-gray-100 rounded-full mb-2">
                                                <span className="text-lg">📜</span>
                                            </div>
                                            <p className="text-gray-600 text-sm">No transactions yet! Make a move to see some action!</p>
                                        </div>
                                    )}
                                </div>

                                {/* Quick Actions */}
                                <div className="bg-white rounded-lg shadow-sm border border-gray-100">
                                    <div className="p-4">
                                        <h2 className="text-lg font-semibold text-gray-900 mb-3">Quick Actions</h2>
                                        <div className="grid grid-cols-2 sm:grid-cols-4 gap-3">
                                            {[
                                                { to: "/top-up", label: "Top Up", icon: "💰", gradient: "from-green-50 to-emerald-50", border: "border-green-100 hover:border-green-200" },
                                                { to: "/transfer", label: "Transfer", icon: "💸", gradient: "from-blue-50 to-indigo-50", border: "border-blue-100 hover:border-blue-200" },
                                                { to: "/withdraw", label: "Withdraw", icon: "🏦", gradient: "from-purple-50 to-pink-50", border: "border-purple-100 hover:border-purple-200" },
                                                { to: "/convert", label: "Convert", icon: "🔄", gradient: "from-blue-50 to-indigo-50", border: "border-blue-100 hover:border-blue-200" },
                                            ].map((action) => (
                                                <Link
                                                    key={action.to}
                                                    to={action.to}
                                                    className={`group relative bg-gradient-to-r ${action.gradient} rounded-lg p-3 border ${action.border} transition-all duration-200 overflow-hidden`}
                                                    aria-label={action.label}
                                                >
                                                    <div className="relative z-10 flex items-center space-x-2">
                                                        <div className="w-6 h-6 bg-opacity-50 bg-white rounded-lg flex items-center justify-center">
                                                            <span className="text-base">{action.icon}</span>
                                                        </div>
                                                        <h3 className="text-sm font-semibold text-gray-900">{action.label}</h3>
                                                    </div>
                                                    <div className="absolute bottom-0 right-0 w-12 h-12 bg-gradient-to-r from-white to-transparent rounded-full -mr-6 -mb-6 transform group-hover:scale-110 transition-transform duration-200 opacity-50"></div>
                                                </Link>
                                            ))}
                                        </div>
                                    </div>
                                </div>
                            </div>
                        )}

                        {/* Transaction Details Modal */}
                        {selectedTransaction && (
                            <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" role="dialog" aria-modal="true" aria-labelledby="transaction-modal-title">
                                <div className="bg-white rounded-lg p-6 max-w-sm w-full">
                                    <h3 id="transaction-modal-title" className="text-lg font-semibold text-gray-900 mb-4">Transaction Details</h3>
                                    <div className="space-y-3">
                                        <p className="text-sm text-gray-600"><span className="font-medium">ID:</span> {selectedTransaction.id}</p>
                                        <p className="text-sm text-gray-600"><span className="font-medium">Type:</span> {selectedTransaction.type.replace("_", " ").toUpperCase()}</p>
                                        <p className="text-sm text-gray-600"><span className="font-medium">Amount:</span> {formatBalance(selectedTransaction.amount, selectedTransaction.currency)}</p>
                                        <p className="text-sm text-gray-600"><span className="font-medium">Date:</span> {formatDate(selectedTransaction.created_at)}</p>
                                        <p className="text-sm text-gray-600"><span className="font-medium">Status:</span> {selectedTransaction.status.toUpperCase()}</p>
                                        {selectedTransaction.notes && (
                                            <p className="text-sm text-gray-600"><span className="font-medium">Notes:</span> {selectedTransaction.notes}</p>
                                        )}
                                    </div>
                                    <button
                                        onClick={() => setSelectedTransaction(null)}
                                        className="mt-4 w-full bg-gray-200 text-gray-700 p-2 rounded-lg hover:bg-gray-300 transition-all duration-200"
                                        aria-label="Close transaction details"
                                    >
                                        Close
                                    </button>
                                </div>
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </ErrorBoundary>
    );
}

export default Dashboard;