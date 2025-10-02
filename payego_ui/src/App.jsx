import React, { useState, useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
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
import LandingPage from "./components/LandingPage.jsx";

function App() {
    const [isAuthenticated, setIsAuthenticated] = useState(!!localStorage.getItem('jwt_token'));

    useEffect(() => {
        const token = localStorage.getItem('jwt_token');
        if (token) {
            setIsAuthenticated(true);
        }
    }, []);

    const handleLogout = () => {
        localStorage.removeItem('jwt_token');
        setIsAuthenticated(false);
    };

    return (
        <Router>
            <div className="min-h-screen bg-gradient-to-br from-blue-50 to-indigo-100">
                <nav className="bg-white shadow-lg border-b border-gray-200">
                    <div className="container mx-auto px-4 sm:px-6 py-4 flex justify-between items-center">
                        <div className="flex items-center space-x-2">
                            <div className="w-8 h-8 bg-gradient-to-r from-blue-600 to-indigo-600 rounded-lg flex items-center justify-center">
                                <span className="text-white font-bold text-sm">P</span>
                            </div>
                            <h1 className="text-xl sm:text-2xl font-bold bg-gradient-to-r from-blue-600 to-indigo-600 bg-clip-text text-transparent">
                                Payego
                            </h1>
                        </div>
                        {isAuthenticated && (
                            <button 
                                onClick={handleLogout} 
                                className="px-4 sm:px-6 py-2 text-sm sm:text-base bg-red-500 text-white rounded-lg hover:bg-red-600 transition-colors duration-200 font-medium shadow-md hover:shadow-lg"
                            >
                                Logout
                            </button>
                        )}
                    </div>
                </nav>
                <div className="container mx-auto px-4 sm:px-6 lg:px-8 py-4 sm:py-8">
                <Routes>
                    <Route path="/" element={!isAuthenticated ? <LandingPage /> : <Navigate to="/dashboard" />} />
                    <Route path="/login" element={!isAuthenticated ? <LoginForm setAuth={setIsAuthenticated} /> : <Navigate to="/" />} />
                    <Route path="/register" element={!isAuthenticated ? <RegisterForm setAuth={setIsAuthenticated} /> : <Navigate to="/" />} />
                    <Route path="/dashboard" element={isAuthenticated ? <Dashboard /> : <Navigate to="/login" />} />
                    <Route path="/top-up" element={isAuthenticated ? <TopUpForm /> : <Navigate to="/login" />} />
                    <Route path="/banks" element={isAuthenticated ? <BankList /> : <Navigate to="/login" />} />
                    <Route path="/add-bank" element={isAuthenticated ? <AddBankForm /> : <Navigate to="/login" />} />
                    <Route path="/transfer" element={isAuthenticated ? <TransferForm /> : <Navigate to="/login" />} />
                    <Route path="/withdraw" element={isAuthenticated ? <WithdrawForm /> : <Navigate to="/login" />} />
                    <Route path="/convert" element={isAuthenticated ? <ConvertForm /> : <Navigate to="/login" />} />
                    <Route path="/success" element={<SuccessPage />} />
                </Routes>
                </div>
            </div>
        </Router>
    );
}

export default App;
