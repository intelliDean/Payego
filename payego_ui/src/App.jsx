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
            <div className="min-h-screen bg-gray-100">
                <nav className="bg-blue-600 text-white p-4">
                    <div className="container mx-auto flex justify-between">
                        <h1 className="text-xl font-bold">Payego</h1>
                        {isAuthenticated && (
                            <button onClick={handleLogout} className="px-4 py-2 bg-red-500 rounded hover:bg-red-600">
                                Logout
                            </button>
                        )}
                    </div>
                </nav>
                <Routes>
                    <Route path="/login" element={!isAuthenticated ? <LoginForm setAuth={setIsAuthenticated} /> : <Navigate to="/" />} />
                    <Route path="/register" element={!isAuthenticated ? <RegisterForm setAuth={setIsAuthenticated} /> : <Navigate to="/" />} />
                    <Route path="/" element={isAuthenticated ? <Dashboard /> : <Navigate to="/login" />} />
                    <Route path="/top-up" element={isAuthenticated ? <TopUpForm /> : <Navigate to="/login" />} />
                    <Route path="/banks" element={isAuthenticated ? <BankList /> : <Navigate to="/login" />} />
                    <Route path="/add-bank" element={isAuthenticated ? <AddBankForm /> : <Navigate to="/login" />} />
                    <Route path="/transfer" element={isAuthenticated ? <TransferForm /> : <Navigate to="/login" />} />
                    <Route path="/withdraw" element={isAuthenticated ? <WithdrawForm /> : <Navigate to="/login" />} />
                    <Route path="/convert" element={isAuthenticated ? <ConvertForm /> : <Navigate to="/login" />} />
                    <Route path="/success" element={<SuccessPage />} />
                </Routes>
            </div>
        </Router>
    );
}

export default App;