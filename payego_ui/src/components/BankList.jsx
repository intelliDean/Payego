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
        <div className="max-w-4xl mx-auto mt-10">
            <div className="mb-8">
                <h1 className="text-4xl font-bold text-gray-800 mb-2">Bank Management</h1>
                <p className="text-gray-600">Manage your connected bank accounts</p>
            </div>
            
            <div className="bg-white rounded-2xl shadow-xl border border-gray-100 p-8">
                <div className="flex justify-between items-center mb-6">
                    <h2 className="text-2xl font-bold text-gray-800">Available Banks</h2>
                    <button
                        onClick={handleInitBanks}
                        disabled={loading}
                        className="bg-gradient-to-r from-green-500 to-emerald-500 text-white px-6 py-3 rounded-lg hover:from-green-600 hover:to-emerald-600 disabled:from-gray-400 disabled:to-gray-400 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                    >
                        {loading ? 'Initializing...' : 'Initialize Banks'}
                    </button>
                </div>
                
                {loading && (
                    <div className="flex justify-center items-center py-12">
                        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-green-600"></div>
                    </div>
                )}
                
                {error && (
                    <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
                        <p className="text-red-600 text-center">{error}</p>
                    </div>
                )}
                
                {!loading && banks.length === 0 && !error && (
                    <div className="text-center py-12">
                        <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
                            <span className="text-gray-400 text-2xl">🏦</span>
                        </div>
                        <p className="text-gray-600 mb-4">No banks available. Initialize banks to get started.</p>
                    </div>
                )}
                
                {!loading && banks.length > 0 && (
                <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mb-6">
                {banks.map((bank) => (
                    <div key={bank.id} className="bg-gradient-to-r from-blue-50 to-indigo-50 rounded-xl p-4 border border-blue-100 hover:from-blue-100 hover:to-indigo-100 transition-all duration-200">
                        <div className="flex items-center space-x-3">
                            <div className="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
                                <span className="text-blue-600 font-bold text-sm">{bank.code}</span>
                            </div>
                            <div>
                                <p className="font-semibold text-gray-800">{bank.name}</p>
                                <p className="text-sm text-gray-600">Code: {bank.code}</p>
                            </div>
                        </div>
                    </div>
                ))}
                </div>
                )}
                
                <div className="text-center pt-6 border-t border-gray-200">
                    <Link 
                        to="/add-bank" 
                        className="inline-flex items-center space-x-2 bg-gradient-to-r from-teal-500 to-cyan-500 text-white px-6 py-3 rounded-lg hover:from-teal-600 hover:to-cyan-600 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                    >
                        <span className="text-xl">+</span>
                        <span>Add Bank Account</span>
                    </Link>
                </div>
            </div>
        </div>
    );
}

export default BankList;