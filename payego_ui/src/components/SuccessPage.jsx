import React, { useState, useEffect } from "react";
import { Link, useSearchParams, useNavigate } from "react-router-dom";
import axios from "axios";
import ErrorBoundary from "./ErrorBoundary";

function SuccessPage() {
  const [searchParams] = useSearchParams();
  const transactionId = searchParams.get("transaction_id");
  const [transaction, setTransaction] = useState(null);
  const [error, setError] = useState(null);
  const [loading, setLoading] = useState(true);
  const navigate = useNavigate();

  useEffect(() => {
    const fetchTransaction = async () => {
      try {
        const token =
          localStorage.getItem("jwt_token") ||
          sessionStorage.getItem("jwt_token");
        if (!token) {
          setError("No session found. Time to join the Payego party!");
          navigate("/login");
          return;
        }

        if (!transactionId) {
          setError("No transaction ID provided. Did it sneak away?");
          setLoading(false);
          return;
        }

        console.log("SuccessPage transactionId:", transactionId);
        await axios.get(`${import.meta.env.VITE_API_URL}/api/current_user`, {
          headers: { Authorization: `Bearer ${token}` },
        });

          console.log('Fetching transaction:', transactionId);  ///api/get_transactions
          const response = await axios.post(
              `${import.meta.env.VITE_API_URL}/api/get_transactions`,
              { txn_id },
              { headers: { Authorization: `Bearer ${token}` } }
          );
        console.log("Transaction details:", response.data);
        setTransaction(response.data);
        setLoading(false);
      } catch (err) {
        console.error("Transaction fetch error:", err);
        if (err.response?.status === 401) {
          setError("Session expired. Back to the login gate!");
          localStorage.removeItem("jwt_token");
          sessionStorage.removeItem("jwt_token");
          navigate("/login");
        } else if (err.response?.status === 404) {
          setError("Transaction not found. Did it vanish into thin air?");
        } else {
          setError(
            err.response?.data?.message ||
              "Transaction details got lost in the void!"
          );
        }
        setLoading(false);
      }
    };
    fetchTransaction();
  }, [transactionId, navigate]);

  const formatAmount = (amount, currency) =>
    new Intl.NumberFormat("en-US", {
      style: "currency",
      currency: currency || "USD",
      minimumFractionDigits: 2,
    }).format((amount || 0) / 100);

  const formatDate = (dateStr) =>
    dateStr
      ? new Date(dateStr).toLocaleDateString("en-US", {
          month: "short",
          day: "numeric",
          year: "numeric",
          hour: "2-digit",
          minute: "2-digit",
        })
      : "N/A";

  return (
    <ErrorBoundary>
      <div className="min-h-screen bg-gray-50 flex items-center justify-center p-4">
        <div className="max-w-md mx-auto p-8 bg-white rounded-2xl shadow-xl border border-gray-100 text-center">
          {loading ? (
            <div className="flex flex-col items-center justify-center py-8">
              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
              <p className="mt-2 text-gray-600 text-sm">
                Fetching transaction details...
              </p>
            </div>
          ) : error ? (
            <div className="space-y-4">
              <div className="w-20 h-20 bg-gradient-to-r from-red-500 to-pink-500 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg
                  className="w-10 h-10 text-white"
                  viewBox="0 0 24 24"
                  fill="currentColor"
                  aria-hidden="true"
                >
                  <path
                    fillRule="evenodd"
                    d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm0-14c-3.31 0-6 2.69-6 6s2.69 6 6 6 6-2.69 6-6-2.69-6-6-6zm0 10c-2.21 0-4-1.79-4-4s1.79-4 4-4 4 1.79 4 4-1.79 4-4 4z"
                    clipRule="evenodd"
                  />
                </svg>
              </div>
              <h2 className="text-2xl font-semibold text-gray-900 mb-2">
                Oops, Something Went Wrong!
              </h2>
              <p className="text-red-600 text-sm mb-4" id="error-message">
                {error}
              </p>
              <Link
                to="/dashboard"
                className="inline-block bg-gradient-to-r from-blue-600 to-indigo-600 text-white px-8 py-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                aria-label="Return to dashboard"
              >
                Return to Dashboard
              </Link>
            </div>
          ) : transaction ? (
            <>
              <div className="mb-8">
                <div className="w-20 h-20 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-full flex items-center justify-center mx-auto mb-4">
                  <svg
                    className="w-10 h-10 text-white"
                    viewBox="0 0 24 24"
                    fill="currentColor"
                    aria-hidden="true"
                  >
                    <path
                      fillRule="evenodd"
                      d="M20.707 5.293a1 1 0 0 0-1.414 0L9 15.586l-4.293-4.293a1 1 0 0 0-1.414 1.414l5 5a1 1 0 0 0 1.414 0l11-11a1 1 0 0 0 0-1.414z"
                      clipRule="evenodd"
                    />
                  </svg>
                </div>
                <h2 className="text-3xl font-bold text-gray-900 mb-2">
                  Payment Successful!
                </h2>
                <p className="text-gray-600 text-sm">
                  Your wallet’s feeling heavier!
                </p>
              </div>
              <div className="mb-6 p-4 bg-gray-50 rounded-lg space-y-2">
                <div className="flex justify-between">
                  <p className="text-sm text-gray-600">Transaction ID</p>
                  <p className="font-mono text-sm text-gray-800 break-all">
                    {transaction.id}
                  </p>
                </div>
                <div className="flex justify-between">
                  <p className="text-sm text-gray-600">Type</p>
                  <p className="text-sm text-gray-800 capitalize">
                    {transaction.type.replace("_", " ")}
                  </p>
                </div>
                <div className="flex justify-between">
                  <p className="text-sm text-gray-600">Amount</p>
                  <p className="text-sm text-gray-800">
                    {formatAmount(transaction.amount, transaction.currency)}
                  </p>
                </div>
                <div className="flex justify-between">
                  <p className="text-sm text-gray-600">Date</p>
                  <p className="text-sm text-gray-800">
                    {formatDate(transaction.created_at)}
                  </p>
                </div>
                <div className="flex justify-between">
                  <p className="text-sm text-gray-600">Status</p>
                  <p className="text-sm text-gray-800 capitalize">
                    {transaction.status}
                  </p>
                </div>
                {transaction.notes && (
                  <div className="flex justify-between">
                    <p className="text-sm text-gray-600">Notes</p>
                    <p className="text-sm text-gray-800 max-w-[200px] truncate">
                      {transaction.notes}
                    </p>
                  </div>
                )}
              </div>
              <Link
                to="/dashboard"
                className="inline-block bg-gradient-to-r from-blue-600 to-indigo-600 text-white px-8 py-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                aria-label="Return to dashboard"
              >
                Return to Dashboard
              </Link>
            </>
          ) : (
            <div className="space-y-4">
              <div className="w-20 h-20 bg-gradient-to-r from-red-500 to-pink-500 rounded-full flex items-center justify-center mx-auto mb-4">
                <svg
                  className="w-10 h-10 text-white"
                  viewBox="0 0 24 24"
                  fill="currentColor"
                  aria-hidden="true"
                >
                  <path
                    fillRule="evenodd"
                    d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm0 18c-4.41 0-8-3.59-8-8s3.59-8 8-8 8 3.59 8 8-3.59 8-8 8zm0-14c-3.31 0-6 2.69-6 6s2.69 6 6 6 6-2.69 6-6-2.69-6-6-6zm0 10c-2.21 0-4-1.79-4-4s1.79-4 4-4 4 1.79 4 4-1.79 4-4 4z"
                    clipRule="evenodd"
                  />
                </svg>
              </div>
              <h2 className="text-2xl font-semibold text-gray-900 mb-2">
                No Transaction Found
              </h2>
              <p className="text-red-600 text-sm mb-4">
                No transaction data available. Did it get lost in the Payego party?
              </p>
              <Link
                to="/dashboard"
                className="inline-block bg-gradient-to-r from-blue-600 to-indigo-600 text-white px-8 py-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                aria-label="Return to dashboard"
              >
                Return to Dashboard
              </Link>
            </div>
          )}
        </div>
      </div>
    </ErrorBoundary>
  );
}

export default SuccessPage;
