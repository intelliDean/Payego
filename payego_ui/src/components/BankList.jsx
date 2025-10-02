import React, { useState, useEffect } from "react";
import axios from "axios";
import { Link, useNavigate } from "react-router-dom";
import ErrorBoundary from "./ErrorBoundary";

function BankList() {
    const [banks, setBanks] = useState([]);
    const [filteredBanks, setFilteredBanks] = useState([]);
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(true);
    const [deleteModal, setDeleteModal] = useState({ open: false, bankId: null, bankName: "" });
    const [searchQuery, setSearchQuery] = useState("");
    const [currentPage, setCurrentPage] = useState(1);
    const [sortField, setSortField] = useState("name");
    const [sortOrder, setSortOrder] = useState("asc");
    const banksPerPage = 6;
    const navigate = useNavigate();

    useEffect(() => {
        const fetchBanks = async () => {
            try {
                const token = localStorage.getItem('jwt_token') || sessionStorage.getItem('jwt_token');
                if (!token) {
                    setError('No session found. Time to log in for the Payego party!');
                    navigate('/login');
                    return;
                }

                const response = await axios.get(
                    `${import.meta.env.VITE_API_URL}/api/banks`,
                    {
                        headers: { 'Authorization': `Bearer ${token}` },
                    }
                );
                console.log("Fetched banks:", response.data.banks); // Debug log
                setBanks(response.data.banks || []); // Ensure banks is an array
            } catch (err) {
                if (err.response?.status === 401) {
                    setError('Session expired. Back to the login gate!');
                    localStorage.removeItem('jwt_token');
                    sessionStorage.removeItem('jwt_token');
                    navigate('/login');
                } else {
                    setError(err.response?.data?.message || 'Banks pulled a vanishing act! Try again!');
                }
            } finally {
                setLoading(false);
            }
        };
        fetchBanks();
    }, [navigate]);

    // Combined search and sort
    useEffect(() => {
        let updatedBanks = [...banks];

        // Apply search filter with defensive checks
        if (searchQuery) {
            updatedBanks = updatedBanks.filter(bank => {
                const name = bank.name && typeof bank.name === 'string' ? bank.name.toLowerCase() : '';
                const accountNumber = bank.account_number && typeof bank.account_number === 'string' ? bank.account_number : '';
                return name.includes(searchQuery.toLowerCase()) || accountNumber.includes(searchQuery);
            });
        }

        // Apply sorting with defensive checks
        updatedBanks.sort((a, b) => {
            const fieldA = (a[sortField] && typeof a[sortField] === 'string' ? a[sortField].toLowerCase() : a[sortField]) || '';
            const fieldB = (b[sortField] && typeof b[sortField] === 'string' ? b[sortField].toLowerCase() : b[sortField]) || '';
            if (sortOrder === "asc") {
                return fieldA > fieldB ? 1 : -1;
            }
            return fieldA < fieldB ? 1 : -1;
        });

        setFilteredBanks(updatedBanks);
        setCurrentPage(1); // Reset to first page on search or sort
    }, [banks, searchQuery, sortField, sortOrder]);

    const handleDelete = async () => {
        setLoading(true);
        setError(null);

        try {
            const token = localStorage.getItem('jwt_token') || sessionStorage.getItem('jwt_token');
            await axios.delete(
                `${import.meta.env.VITE_API_URL}/api/delete_bank/${deleteModal.bankId}`,
                {
                    headers: { 'Authorization': `Bearer ${token}` },
                }
            );
            setBanks(banks.filter(bank => bank.id !== deleteModal.bankId));
            setDeleteModal({ open: false, bankId: null, bankName: "" });
        } catch (err) {
            if (err.response?.status === 401) {
                setError('Session expired. Back to the login gate!');
                localStorage.removeItem('jwt_token');
                sessionStorage.removeItem('jwt_token');
                navigate('/login');
            } else {
                setError(err.response?.data?.message || 'Failed to kick that bank out!');
            }
        } finally {
            setLoading(false);
        }
    };

    const openDeleteModal = (bankId, bankName) => {
        setDeleteModal({ open: true, bankId, bankName });
    };

    const closeDeleteModal = () => {
        setDeleteModal({ open: false, bankId: null, bankName: "" });
    };

    // Pagination
    const indexOfLastBank = currentPage * banksPerPage;
    const indexOfFirstBank = indexOfLastBank - banksPerPage;
    const currentBanks = filteredBanks.slice(indexOfFirstBank, indexOfLastBank);
    const totalPages = Math.ceil(filteredBanks.length / banksPerPage);

    const handlePrevious = () => {
        if (currentPage > 1) {
            setCurrentPage(currentPage - 1);
        }
    };

    const handleNext = () => {
        if (currentPage < totalPages) {
            setCurrentPage(currentPage + 1);
        }
    };

    const handleSort = (field) => {
        if (sortField === field) {
            setSortOrder(sortOrder === "asc" ? "desc" : "asc");
        } else {
            setSortField(field);
            setSortOrder("asc");
        }
    };

    return (
        <ErrorBoundary>
            <div className="max-w-4xl mx-auto mt-10 p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
                <div className="mb-8">
                    <h1 className="text-4xl font-bold text-gray-800 mb-2">Bank Management</h1>
                    <p className="text-gray-600">Manage your connected bank accounts</p>
                </div>

                <div className="mb-6 flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                    <div className="flex-1">
                        <label htmlFor="search" className="sr-only">Search banks</label>
                        <input
                            id="search"
                            type="text"
                            value={searchQuery}
                            onChange={(e) => setSearchQuery(e.target.value)}
                            placeholder="Search by bank name or account number"
                            className="w-full p-3 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent transition-all duration-200"
                            aria-label="Search banks"
                        />
                    </div>
                    <div className="flex items-center space-x-4">
                        <button
                            onClick={() => handleSort("name")}
                            className="text-blue-600 hover:text-blue-700 font-medium transition-colors duration-200"
                            aria-label={`Sort by bank name (${sortField === "name" ? sortOrder : "asc"})`}
                        >
                            Sort by Name {sortField === "name" && (sortOrder === "asc" ? "↑" : "↓")}
                        </button>
                        <button
                            onClick={() => handleSort("account_number")}
                            className="text-blue-600 hover:text-blue-700 font-medium transition-colors duration-200"
                            aria-label={`Sort by account number (${sortField === "account_number" ? sortOrder : "asc"})`}
                        >
                            Sort by Account {sortField === "account_number" && (sortOrder === "asc" ? "↑" : "↓")}
                        </button>
                    </div>
                </div>

                {loading && (
                    <div className="flex flex-col items-center justify-center py-12">
                        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                        <p className="mt-4 text-gray-600">Fetching your banks...</p>
                    </div>
                )}

                {error && (
                    <div className="mb-6 p-4 bg-red-50 border border-red-200 rounded-lg">
                        <p className="text-red-600 text-center text-sm">{error}</p>
                    </div>
                )}

                {!loading && filteredBanks.length === 0 && !error && (
                    <div className="text-center py-12">
                        <div className="w-16 h-16 bg-gray-100 rounded-full flex items-center justify-center mx-auto mb-4">
                            <span className="text-gray-400 text-2xl">🏦</span>
                        </div>
                        <p className="text-gray-600 mb-4">No banks yet! Add one to join the Payego party!</p>
                    </div>
                )}

                {!loading && currentBanks.length > 0 && (
                    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4 mb-6">
                        {currentBanks.map((bank) => (
                            <div
                                key={bank.id}
                                className="bg-gradient-to-r from-blue-50 to-indigo-50 rounded-xl p-4 border border-blue-100 hover:from-blue-100 hover:to-indigo-100 transition-all duration-200"
                                role="region"
                                aria-label={`Bank account: ${bank.name}`}
                            >
                                <div className="flex items-center justify-between">
                                    <div className="flex items-center space-x-3">
                                        <div className="w-10 h-10 bg-blue-100 rounded-lg flex items-center justify-center">
                                            <span className="text-blue-600 font-bold text-sm">
                                                {bank.code}
                                            </span>
                                        </div>
                                        <div>
                                            <p className="font-semibold text-gray-800">{bank.name}</p>
                                            <p className="text-sm text-gray-600">Account: {bank.account_number}</p>
                                            <p className="text-sm text-gray-600">Code: {bank.code}</p>
                                        </div>
                                    </div>
                                    <button
                                        onClick={() => openDeleteModal(bank.id, bank.name)}
                                        className="text-red-600 hover:text-red-700 font-medium transition-colors duration-200"
                                        aria-label={`Delete ${bank.name} account`}
                                    >
                                        Delete
                                    </button>
                                </div>
                            </div>
                        ))}
                    </div>
                )}

                {!loading && filteredBanks.length > banksPerPage && (
                    <div className="flex flex-col sm:flex-row justify-center items-center gap-2 sm:gap-4 mt-6">
                        <button
                            onClick={handlePrevious}
                            disabled={currentPage === 1}
                            className="w-full sm:w-auto px-4 py-2 rounded-lg font-medium transition-all duration-200 bg-gray-200 text-gray-700 hover:bg-gray-300 disabled:bg-gray-100 disabled:text-gray-400 text-sm sm:text-base"
                            aria-label="Go to previous page"
                        >
                            ← Previous
                        </button>
                        <span className="px-4 py-2 text-gray-600 text-sm sm:text-base">
                            Page {currentPage} of {totalPages}
                        </span>
                        <button
                            onClick={handleNext}
                            disabled={currentPage === totalPages}
                            className="w-full sm:w-auto px-4 py-2 rounded-lg font-medium transition-all duration-200 bg-gray-200 text-gray-700 hover:bg-gray-300 disabled:bg-gray-100 disabled:text-gray-400 text-sm sm:text-base"
                            aria-label="Go to next page"
                        >
                            Next →
                        </button>
                    </div>
                )}

                <div className="text-center pt-6 border-t border-gray-200">
                    <Link
                        to="/add-bank"
                        className="inline-flex items-center space-x-2 bg-gradient-to-r from-blue-600 to-indigo-600 text-white px-6 py-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                        aria-label="Add new bank account"
                    >
                        <span className="text-xl">+</span>
                        <span>Add Bank Account</span>
                    </Link>
                </div>

                {deleteModal.open && (
                    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50" role="dialog" aria-modal="true" aria-labelledby="delete-modal-title">
                        <div className="bg-white rounded-2xl p-6 max-w-sm w-full">
                            <h3 id="delete-modal-title" className="text-xl font-bold text-gray-800 mb-4">Confirm Delete</h3>
                            <p className="text-gray-600 mb-6">
                                Are you sure you want to delete your {deleteModal.bankName} account?
                            </p>
                            <div className="flex space-x-4">
                                <button
                                    onClick={handleDelete}
                                    disabled={loading}
                                    className="flex-1 bg-red-600 text-white p-3 rounded-lg hover:bg-red-700 disabled:bg-gray-400 transition-all duration-200 font-medium"
                                    aria-label={`Confirm delete ${deleteModal.bankName} account`}
                                >
                                    {loading ? 'Deleting...' : 'Delete'}
                                </button>
                                <button
                                    onClick={closeDeleteModal}
                                    className="flex-1 bg-gray-200 text-gray-700 p-3 rounded-lg hover:bg-gray-300 transition-all duration-200 font-medium"
                                    aria-label="Cancel delete"
                                >
                                    Cancel
                                </button>
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </ErrorBoundary>
    );
}

export default BankList;
