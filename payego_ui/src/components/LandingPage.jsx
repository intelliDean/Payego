import React from 'react';
import { useNavigate } from 'react-router-dom';
import ErrorBoundary from './ErrorBoundary';

function LandingPage() {
    const navigate = useNavigate();

    const handleSignUp = () => {
        navigate('/signup');
    };

    const handleLogin = () => {
        navigate('/login');
    };

    return (
        <ErrorBoundary>
            <div className="bg-white min-h-screen">
                {/* Hero Section */}
                <section className="max-w-md mx-auto mt-10 p-8 bg-white rounded-2xl shadow-xl border border-gray-100 text-center">
                    <div className="w-16 h-16 bg-gradient-to-r from-purple-500 to-pink-500 rounded-2xl flex items-center justify-center mx-auto mb-4">
                        <span className="text-white text-2xl">💸</span>
                    </div>
                    <h1 className="text-4xl font-bold text-gray-800 mb-4">Welcome to Payego</h1>
                    <p className="text-gray-600 text-lg mb-6">
                        Your wallet, your rules! Top up, withdraw, and transfer funds faster than you can say "Show me the money!" 🏦
                    </p>
                    <div className="flex space-x-4">
                        <button
                            onClick={handleSignUp}
                            className="flex-1 bg-gradient-to-r from-purple-500 to-pink-500 text-white p-3 rounded-lg hover:from-purple-600 hover:to-pink-600 transition-all duration-200 font-medium shadow-lg hover:shadow-xl transform hover:-translate-y-0.5"
                            aria-label="Sign up for Payego"
                        >
                            Join the Party
                        </button>
                        <button
                            onClick={handleLogin}
                            className="flex-1 bg-gray-200 text-gray-700 p-3 rounded-lg hover:bg-gray-300 transition-all duration-200 font-medium"
                            aria-label="Log in to Payego"
                        >
                            Log In
                        </button>
                    </div>
                </section>

                {/* Features Section */}
                <section className="max-w-md mx-auto mt-10 p-8">
                    <h2 className="text-3xl font-bold text-gray-800 text-center mb-8">Why Payego?</h2>
                    <div className="space-y-6">
                        <div className="bg-white rounded-2xl shadow-md p-6 border border-gray-100">
                            <div className="w-12 h-12 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg flex items-center justify-center mb-4">
                                <span className="text-white text-xl">💰</span>
                            </div>
                            <h3 className="text-xl font-semibold text-gray-800 mb-2">Top Up Your Wallet</h3>
                            <p className="text-gray-600">
                                Add funds with Stripe or PayPal. It’s so easy, even your grandma could do it! 😎
                            </p>
                            <button
                                onClick={() => navigate('/top-up')}
                                className="mt-4 text-purple-500 hover:text-purple-600 font-medium"
                                aria-label="Learn more about topping up"
                            >
                                Get Started →
                            </button>
                        </div>
                        <div className="bg-white rounded-2xl shadow-md p-6 border border-gray-100">
                            <div className="w-12 h-12 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg flex items-center justify-center mb-4">
                                <span className="text-white text-xl">🏦</span>
                            </div>
                            <h3 className="text-xl font-semibold text-gray-800 mb-2">Withdraw to Bank</h3>
                            <p className="text-gray-600">
                                Cash out to your bank account faster than you can say "Cha-ching!" 💸
                            </p>
                            <button
                                onClick={() => navigate('/withdraw')}
                                className="mt-4 text-purple-500 hover:text-purple-600 font-medium"
                                aria-label="Learn more about withdrawing"
                            >
                                Get Started →
                            </button>
                        </div>
                        <div className="bg-white rounded-2xl shadow-md p-6 border border-gray-100">
                            <div className="w-12 h-12 bg-gradient-to-r from-purple-500 to-pink-500 rounded-lg flex items-center justify-center mb-4">
                                <span className="text-white text-xl">🔄</span>
                            </div>
                            <h3 className="text-xl font-semibold text-gray-800 mb-2">Transfer & Convert</h3>
                            <p className="text-gray-600">
                                Move money between currencies like a financial ninja. USD to NGN? No problem! 🥷
                            </p>
                            <button
                                onClick={() => navigate('/transfer')}
                                className="mt-4 text-purple-500 hover:text-purple-600 font-medium"
                                aria-label="Learn more about transferring"
                            >
                                Get Started →
                            </button>
                        </div>
                    </div>
                </section>

                {/* Footer */}
                <footer className="max-w-md mx-auto mt-10 p-8 text-center border-t border-gray-200">
                    <p className="text-gray-600 text-sm mb-4">
                        Payego: Where your money parties hard and stays safe! 🎉
                    </p>
                    <div className="flex justify-center space-x-4">
                        <a href="/privacy" className="text-purple-500 hover:text-purple-600 text-sm">
                            Privacy Policy
                        </a>
                        <a href="/terms" className="text-purple-500 hover:text-purple-600 text-sm">
                            Terms of Service
                        </a>
                        <a href="/contact" className="text-purple-500 hover:text-purple-600 text-sm">
                            Contact Us
                        </a>
                    </div>
                    <p className="text-gray-500 text-sm mt-4">© 2025 Payego. All rights reserved.</p>
                </footer>
            </div>
        </ErrorBoundary>
    );
}

export default LandingPage;
