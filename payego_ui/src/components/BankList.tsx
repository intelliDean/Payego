import React, { useState, useMemo } from "react";
import { Link } from "react-router-dom";
import ErrorBoundary from "./ErrorBoundary";
import { useUserBankAccounts } from "../hooks/useBanks";
import { bankApi } from "../api/bank";
import { useQueryClient } from "@tanstack/react-query";

const BankList: React.FC = () => {
    const { data: banks, isLoading } = useUserBankAccounts();
    const queryClient = useQueryClient();

    const [deleteModal, setDeleteModal] = useState<{ open: boolean, bankId: string | null, bankName: string }>({
        open: false, bankId: null, bankName: ""
    });
    const [searchQuery, setSearchQuery] = useState("");

    const filteredBanks = useMemo(() => {
        if (!banks) return [];
        let result = [...banks];

        if (searchQuery) {
            const query = searchQuery.toLowerCase();
            result = result.filter(b =>
                b.bank_name.toLowerCase().includes(query) ||
                b.account_number.includes(query)
            );
        }

        return result;
    }, [banks, searchQuery]);

    const handleDelete = async () => {
        if (!deleteModal.bankId) return;
        try {
            await bankApi.deleteBankAccount(deleteModal.bankId);
            queryClient.invalidateQueries({ queryKey: ['user-banks'] });
            setDeleteModal({ open: false, bankId: null, bankName: "" });
        } catch (err) {
            console.error("Failed to delete bank:", err);
        }
    };



    return (
        <ErrorBoundary>
            <div className="max-w-4xl mx-auto mt-10 p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
                <div className="flex justify-between items-center mb-8">
                    <div>
                        <h1 className="text-4xl font-bold text-gray-800 mb-2">Bank Management</h1>
                        <p className="text-gray-600">Manage your connected bank accounts</p>
                    </div>
                    <Link to="/add-bank" className="btn-primary px-6 py-3 rounded-lg shadow-lg">+ Add Bank</Link>
                </div>

                <div className="mb-6 flex gap-4">
                    <input
                        type="text"
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        placeholder="Search by bank name or account number"
                        className="flex-1 p-3 border border-gray-300 rounded-lg"
                    />
                </div>

                {isLoading ? (
                    <div className="flex justify-center py-12">
                        <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-blue-600"></div>
                    </div>
                ) : (
                    <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        {filteredBanks.map((bank) => (
                            <div key={bank.id} className="bg-gray-50 p-4 rounded-xl border flex justify-between items-center">
                                <div>
                                    <p className="font-bold text-gray-800">{bank.bank_name}</p>
                                    <p className="text-sm text-gray-600">Acc: {bank.account_number}</p>
                                </div>
                                <button
                                    onClick={() => setDeleteModal({ open: true, bankId: bank.id, bankName: bank.bank_name })}
                                    className="text-red-600 hover:underline text-sm font-medium"
                                >
                                    Delete
                                </button>
                            </div>
                        ))}
                    </div>
                )}

                {deleteModal.open && (
                    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
                        <div className="bg-white p-6 rounded-2xl max-w-sm w-full shadow-2xl">
                            <h3 className="text-xl font-bold mb-4 text-gray-900">Confirm Action</h3>
                            <p className="text-gray-600 mb-6">Remove {deleteModal.bankName}?</p>
                            <div className="flex gap-4">
                                <button onClick={handleDelete} className="flex-1 bg-red-600 text-white p-3 rounded-lg font-bold">Remove</button>
                                <button onClick={() => setDeleteModal({ open: false, bankId: null, bankName: "" })} className="flex-1 bg-gray-200 p-3 rounded-lg font-bold">Cancel</button>
                            </div>
                        </div>
                    </div>
                )}
            </div>
        </ErrorBoundary>
    );
};

export default BankList;
