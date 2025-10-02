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
        console.log("JWT Token:", token);
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
        console.log("API URL:", `${import.meta.env.VITE_API_URL}/api/transactions/${transactionId}`);

        await axios.get(`${import.meta.env.VITE_API_URL}/api/current_user`, {
          headers: { Authorization: `Bearer ${token}` },
        });

        console.log('Fetching transaction:', transactionId);
        const response = await axios.get(`${import.meta.env.VITE_API_URL}/api/transactions/${transactionId}`, {
          headers: { Authorization: `Bearer ${token}` },
        });

        console.log("Transaction details:", response.data);
        setTransaction(response.data);
        setLoading(false);
      } catch (err) {
        console.error("Transaction fetch error:", err);
        console.error("Error response:", err.response?.data);
        console.error("Error status:", err.response?.status);
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
        <div className="min-h-screen bg-gray-50 flex items-center justify-center p-4 sm:p-6">
          <div className="max-w-md mx-auto p-6 sm:p-8 bg-white rounded-2xl shadow-xl border border-gray-100 text-center">
            {loading ? (
                <div className="flex flex-col items-center justify-center py-8">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-blue-600"></div>
                  <p className="mt-2 text-gray-600 text-sm sm:text-base">
                    Fetching transaction details...
                  </p>
                </div>
            ) : error ? (
                <div className="space-y-4">
                  <div className="w-16 sm:w-20 h-16 sm:h-20 bg-gradient-to-r from-red-500 to-pink-500 rounded-full flex items-center justify-center mx-auto mb-4">
                    <svg
                        className="w-8 sm:w-10 h-8 sm:h-10 text-white"
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
                  <h2 className="text-xl sm:text-2xl font-semibold text-gray-900 mb-2">
                    Oops, Something Went Wrong!
                  </h2>
                  <p className="text-red-600 text-sm sm:text-base mb-4" id="error-message">
                    {error}
                  </p>
                  <Link
                      to="/"
                      className="inline-block bg-gradient-to-r from-blue-600 to-indigo-600 text-white px-6 sm:px-8 py-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5 text-sm sm:text-base"
                      aria-label="Return to dashboard"
                  >
                    Return to Dashboard
                  </Link>
                </div>
            ) : transaction ? (
                <>
                  <div className="mb-8">
                    <div className="w-16 sm:w-20 h-16 sm:h-20 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-full flex items-center justify-center mx-auto mb-4">
                      <svg
                          className="w-8 sm:w-10 h-8 sm:h-10 text-white"
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
                    <h2 className="text-2xl sm:text-3xl font-bold text-gray-900 mb-2">
                      Payment Successful!
                    </h2>
                    <p className="text-gray-600 text-sm sm:text-base">
                      Your wallet's feeling heavier!
                    </p>
                  </div>
                  <div className="mb-6 p-4 bg-gray-50 rounded-lg space-y-2 text-left">
                    <div className="flex flex-col sm:flex-row sm:justify-between gap-1 sm:gap-0">
                      <p className="text-sm text-gray-600">Transaction ID</p>
                      <p className="font-mono text-xs sm:text-sm text-gray-800 break-all">
                        {transaction.id}
                      </p>
                    </div>
                    <div className="flex flex-col sm:flex-row sm:justify-between gap-1 sm:gap-0">
                      <p className="text-sm text-gray-600">Type</p>
                      <p className="text-sm text-gray-800 capitalize">
                        {transaction.transaction_type ? transaction.transaction_type.replace("_", " ") : "Unknown"}
                      </p>
                    </div>
                    <div className="flex flex-col sm:flex-row sm:justify-between gap-1 sm:gap-0">
                      <p className="text-sm text-gray-600">Amount</p>
                      <p className="text-sm text-gray-800">
                        {formatAmount(transaction.amount, transaction.currency)}
                      </p>
                    </div>
                    <div className="flex flex-col sm:flex-row sm:justify-between gap-1 sm:gap-0">
                      <p className="text-sm text-gray-600">Date</p>
                      <p className="text-sm text-gray-800">
                        {formatDate(transaction.created_at)}
                      </p>
                    </div>
                    <div className="flex flex-col sm:flex-row sm:justify-between gap-1 sm:gap-0">
                      <p className="text-sm text-gray-600">Status</p>
                      <p className="text-sm text-gray-800 capitalize">
                        {transaction.status}
                      </p>
                    </div>
                    {transaction.notes && (
                        <div className="flex flex-col sm:flex-row sm:justify-between gap-1 sm:gap-0">
                          <p className="text-sm text-gray-600">Notes</p>
                          <p className="text-sm text-gray-800 sm:max-w-[200px] sm:truncate">
                            {transaction.notes}
                          </p>
                        </div>
                    )}
                  </div>
                  <Link
                      to="/"
                      className="inline-block bg-gradient-to-r from-blue-600 to-indigo-600 text-white px-6 sm:px-8 py-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5 text-sm sm:text-base"
                      aria-label="Return to dashboard"
                  >
                    Return to Dashboard
                  </Link>
                </>
            ) : (
                <div className="space-y-4">
                  <div className="w-16 sm:w-20 h-16 sm:h-20 bg-gradient-to-r from-red-500 to-pink-500 rounded-full flex items-center justify-center mx-auto mb-4">
                    <svg
                        className="w-8 sm:w-10 h-8 sm:h-10 text-white"
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
                  <h2 className="text-xl sm:text-2xl font-semibold text-gray-900 mb-2">
                    No Transaction Found
                  </h2>
                  <p className="text-red-600 text-sm sm:text-base mb-4">
                    No transaction data available. Did it get lost in the Payego party?
                  </p>
                  <Link
                      to="/"
                      className="inline-block bg-gradient-to-r from-blue-600 to-indigo-600 text-white px-6 sm:px-8 py-3 rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5 text-sm sm:text-base"
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


//
//
// import React, { useState, useEffect } from 'react';
// import axios from 'axios';
// import { useLocation } from 'react-router-dom';
// import PayPalPayment from './PayPalPayment';
//
// function SuccessPage() {
//     const [transaction, setTransaction] = useState(null);
//     const [error, setError] = useState(null);
//     const [loading, setLoading] = useState(true);
//     const location = useLocation();
//
//     const getToken = () => localStorage.getItem('jwt_token') || sessionStorage.getItem('jwt_token');
//
//     useEffect(() => {
//         const fetchTransaction = async () => {
//             const params = new URLSearchParams(location.search);
//             const transactionId = params.get('transaction_id');
//             if (!transactionId) {
//                 setError('No transaction ID provided. Payego got lost in transit!');
//                 setLoading(false);
//                 return;
//             }
//
//             try {
//                 const token = getToken();
//                 if (!token) {
//                     throw new Error('No session found. Time to join the Payego party!');
//                 }
//                 const response = await axios.get(
//                     `${import.meta.env.VITE_API_URL}/api/transactions/${transactionId}`,
//                     {
//                         headers: {
//                             'Authorization': `Bearer ${token}`,
//                         },
//                     }
//                 );
//                 console.log('Transaction details:', response.data);
//                 setTransaction(response.data);
//             } catch (err) {
//                 console.error('Fetch transaction error:', err);
//                 setError(err.response?.data?.message || 'Transaction fetch crashed the Payego party!');
//             } finally {
//                 setLoading(false);
//             }
//         };
//
//         fetchTransaction();
//     }, [location]);
//
//     const params = new URLSearchParams(location.search);
//     const transactionId = params.get('transaction_id');
//     const provider = transaction?.type?.startsWith('topup_paypal') ? 'paypal' : 'stripe';
//
//     return (
//         <div className="min-h-screen bg-gray-50 flex items-center justify-center p-4">
//             <div className="max-w-md mx-auto p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
//                 <h2 className="text-2xl font-bold text-gray-900 mb-6 text-center">
//                     Payment Status
//                 </h2>
//                 {provider === 'paypal' && !transaction && (
//                     <PayPalPayment transactionId={transactionId} />
//                 )}
//                 {loading && (
//                     <div className="mt-4 p-3 bg-blue-50 border border-blue-200 rounded-lg">
//                         <p className="text-blue-600 text-center text-sm">Loading transaction details...</p>
//                     </div>
//                 )}
//                 {error && (
//                     <div className="mt-4 p-3 bg-red-50 border border-red-200 rounded-lg">
//                         <p className="text-red-600 text-center text-sm">{error}</p>
//                     </div>
//                 )}
//                 {transaction && (
//                     <div className="mt-4 space-y-4">
//                         <p className="text-gray-700">
//                             <span className="font-medium">Status:</span> {transaction.status}
//                         </p>
//                         <p className="text-gray-700">
//                             <span className="font-medium">Type:</span> {transaction.type}
//                         </p>
//                         <p className="text-gray-700">
//                             <span className="font-medium">Amount:</span> {(transaction.amount / 100).toFixed(2)} {transaction.currency}
//                         </p>
//                         <p className="text-gray-700">
//                             <span className="font-medium">Notes:</span> {transaction.description}
//                         </p>
//                         <button
//                             onClick={() => window.location.href = '/dashboard'}
//                             className="w-full bg-gradient-to-r from-blue-600 to-indigo-600 text-white px-4 py-2 rounded-lg hover:from-blue-700 hover:to-indigo-700 transition-all duration-200 font-medium"
//                         >
//                             Back to Dashboard
//                         </button>
//                     </div>
//                 )}
//             </div>
//         </div>
//     );
// }
//
// export default SuccessPage;