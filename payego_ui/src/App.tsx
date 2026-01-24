import React from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { useAuth } from './contexts/AuthContext';
import LoginForm from './components/LoginForm';
import RegisterForm from './components/RegisterForm';
import Dashboard from './components/Dashboard';
import TopUpForm from './components/TopUpForm';
import BankList from './components/BankList';
import AddBankForm from './components/AddBankForm';
import TransferForm from './components/TransferForm';
import WithdrawForm from './components/WithdrawForm';
import ConvertForm from './components/ConvertForm';
import SuccessPage from './components/SuccessPage';
import LandingPage from "./components/LandingPage";

const ProtectedRoute: React.FC<{ children: React.ReactNode }> = ({ children }) => {
    const { isAuthenticated, isLoading } = useAuth();

    if (isLoading) {
        return (
            <div className="min-h-screen flex items-center justify-center">
                <div className="animate-spin rounded-full h-12 w-12 border-b-2 border-purple-600"></div>
            </div>
        );
    }

    return isAuthenticated ? <>{children}</> : <Navigate to="/login" />;
};

function App() {
    const { isAuthenticated, logout } = useAuth();

    return (
        <Router>
            <div className="min-h-screen gradient-bg">
                {/* Modern Navigation Bar */}
                {isAuthenticated && (
                    <nav className="sticky top-0 z-50 glass-strong bg-white/90 shadow-lg border-b border-gray-200/50">
                        <div className="container mx-auto px-4 sm:px-6 py-4">
                            <div className="flex justify-between items-center">
                                {/* Brand */}
                                <div className="flex items-center space-x-3">
                                    <div className="relative group">
                                        <div className="absolute inset-0 bg-gradient-to-r from-purple-600 to-blue-600 rounded-xl blur opacity-50 group-hover:opacity-75 transition-opacity"></div>
                                        <div className="relative w-10 h-10 bg-gradient-to-br from-purple-600 to-blue-600 rounded-xl flex items-center justify-center shadow-lg">
                                            <span className="text-white font-bold text-lg">P</span>
                                        </div>
                                    </div>
                                    <h1 className="text-xl sm:text-2xl font-black gradient-text">
                                        Payego
                                    </h1>
                                </div>

                                {/* Logout Button */}
                                <button
                                    onClick={logout}
                                    className="btn-danger btn-sm flex items-center space-x-2"
                                >
                                    <svg className="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                        <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M17 16l4-4m0 0l-4-4m4 4H7m6 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h4a3 3 0 013 3v1" />
                                    </svg>
                                    <span className="hidden sm:inline">Logout</span>
                                </button>
                            </div>
                        </div>
                    </nav>
                )}

                {/* Main Content */}
                <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-4 sm:py-8">
                    <Routes>
                        <Route path="/" element={!isAuthenticated ? <LandingPage /> : <Navigate to="/dashboard" />} />
                        <Route path="/login" element={!isAuthenticated ? <LoginForm /> : <Navigate to="/" />} />
                        <Route path="/register" element={!isAuthenticated ? <RegisterForm /> : <Navigate to="/" />} />

                        <Route path="/dashboard" element={<ProtectedRoute><Dashboard /></ProtectedRoute>} />
                        <Route path="/top-up" element={<ProtectedRoute><TopUpForm /></ProtectedRoute>} />
                        <Route path="/banks" element={<ProtectedRoute><BankList /></ProtectedRoute>} />
                        <Route path="/add-bank" element={<ProtectedRoute><AddBankForm /></ProtectedRoute>} />
                        <Route path="/transfer" element={<ProtectedRoute><TransferForm /></ProtectedRoute>} />
                        <Route path="/withdraw" element={<ProtectedRoute><WithdrawForm /></ProtectedRoute>} />
                        <Route path="/convert" element={<ProtectedRoute><ConvertForm /></ProtectedRoute>} />
                        <Route path="/success" element={<SuccessPage />} />
                    </Routes>
                </div>
            </div>
        </Router>
    );
}

export default App;
