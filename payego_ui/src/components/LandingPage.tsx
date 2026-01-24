import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import ErrorBoundary from './ErrorBoundary';

const LandingPage: React.FC = () => {
    const navigate = useNavigate();
    const [isVisible, setIsVisible] = useState(false);

    useEffect(() => {
        setIsVisible(true);
    }, []);

    const handleGetStarted = () => navigate('/register');
    const handleLogin = () => navigate('/login');

    const features = [
        {
            icon: '💳',
            title: 'Multi-Currency Wallets',
            description: 'Support for 20+ currencies including USD, EUR, GBP, NGN and more',
            gradient: 'from-blue-500 to-cyan-500',
            delay: '0ms'
        },
        {
            icon: '🚀',
            title: 'Instant Top-ups',
            description: 'Add funds instantly with Stripe or PayPal integration',
            gradient: 'from-purple-500 to-pink-500',
            delay: '100ms'
        },
        {
            icon: '🏦',
            title: 'Bank Withdrawals',
            description: 'Withdraw to your bank account with real-time processing',
            gradient: 'from-green-500 to-emerald-500',
            delay: '200ms'
        },
        {
            icon: '🔄',
            title: 'Currency Exchange',
            description: 'Convert between currencies at competitive rates',
            gradient: 'from-orange-500 to-red-500',
            delay: '300ms'
        },
        {
            icon: '⚡',
            title: 'Lightning Transfers',
            description: 'Send money to other users or external bank accounts instantly',
            gradient: 'from-indigo-500 to-purple-500',
            delay: '400ms'
        },
        {
            icon: '🔒',
            title: 'Bank-Level Security',
            description: 'Your funds are protected with enterprise-grade security',
            gradient: 'from-gray-600 to-gray-800',
            delay: '500ms'
        }
    ];

    const stats = [
        { number: '50K+', label: 'Active Users', icon: '👥' },
        { number: '$10M+', label: 'Processed', icon: '💰' },
        { number: '20+', label: 'Currencies', icon: '🌍' },
        { number: '99.9%', label: 'Uptime', icon: '⚡' }
    ];

    const testimonials = [
        {
            name: 'Sarah Chen',
            role: 'Freelance Designer',
            avatar: 'SC',
            content: 'Payego made managing my international payments so much easier. The currency conversion is seamless!',
            rating: 5
        },
        {
            name: 'Michael Rodriguez',
            role: 'E-commerce Owner',
            avatar: 'MR',
            content: 'Best digital wallet I\'ve used. Fast, secure, and the interface is beautiful.',
            rating: 5
        },
        {
            name: 'Aisha Okonkwo',
            role: 'Remote Developer',
            avatar: 'AO',
            content: 'Finally, a wallet that supports NGN properly. Instant transfers are a game-changer!',
            rating: 5
        }
    ];

    return (
        <ErrorBoundary>
            <div className="min-h-screen gradient-bg">
                {/* Animated Background Elements */}
                <div className="fixed inset-0 overflow-hidden pointer-events-none">
                    <div className="absolute top-20 left-10 w-72 h-72 bg-purple-300 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-float"></div>
                    <div className="absolute top-40 right-10 w-72 h-72 bg-blue-300 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-float" style={{ animationDelay: '1s' }}></div>
                    <div className="absolute -bottom-8 left-1/2 w-72 h-72 bg-pink-300 rounded-full mix-blend-multiply filter blur-xl opacity-20 animate-float" style={{ animationDelay: '2s' }}></div>
                </div>

                {/* Hero Section */}
                <section className="relative overflow-hidden">
                    <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 pt-16 sm:pt-24 pb-16 sm:pb-20">
                        <div className={`text-center transition-all duration-1000 ${isVisible ? 'opacity-100 translate-y-0' : 'opacity-0 translate-y-10'}`}>
                            {/* Logo */}
                            <div className="flex justify-center mb-8">
                                <div className="relative group">
                                    <div className="absolute inset-0 bg-gradient-to-r from-purple-600 to-blue-600 rounded-3xl blur-lg opacity-50 group-hover:opacity-75 transition-opacity duration-300"></div>
                                    <div className="relative w-20 h-20 bg-gradient-to-br from-purple-600 via-blue-600 to-indigo-600 rounded-3xl flex items-center justify-center shadow-2xl transform group-hover:scale-105 transition-transform duration-300">
                                        <span className="text-white font-black text-3xl">P</span>
                                    </div>
                                </div>
                            </div>

                            {/* Headline */}
                            <h1 className="text-4xl sm:text-5xl md:text-6xl lg:text-7xl font-black text-gray-900 mb-6 leading-tight px-4">
                                Your Money,{' '}
                                <span className="relative inline-block">
                                    <span className="gradient-text">Simplified</span>
                                    <svg className="absolute -bottom-2 left-0 w-full" height="12" viewBox="0 0 200 12" fill="none">
                                        <path d="M2 10C50 2 150 2 198 10" stroke="url(#gradient-line)" strokeWidth="3" strokeLinecap="round" />
                                        <defs>
                                            <linearGradient id="gradient-line" x1="0%" y1="0%" x2="100%" y2="0%">
                                                <stop offset="0%" stopColor="#8b5cf6" />
                                                <stop offset="100%" stopColor="#3b82f6" />
                                            </linearGradient>
                                        </defs>
                                    </svg>
                                </span>
                            </h1>

                            <p className="text-lg sm:text-xl md:text-2xl text-gray-600 mb-8 max-w-4xl mx-auto leading-relaxed px-4 font-medium">
                                The modern digital wallet that makes managing multiple currencies as easy as sending a text.
                            </p>

                            <div className="flex flex-col sm:flex-row gap-4 justify-center items-center mb-12 px-4">
                                <button onClick={handleGetStarted} className="w-full sm:w-auto btn-primary-glow btn-lg group">
                                    <span className="flex items-center justify-center space-x-2">
                                        <span>Get Started Free</span>
                                        <svg className="w-5 h-5 group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
                                        </svg>
                                    </span>
                                </button>
                                <button onClick={handleLogin} className="w-full sm:w-auto btn-secondary btn-lg">
                                    Sign In
                                </button>
                            </div>

                            <div className="grid grid-cols-2 md:grid-cols-4 gap-6 max-w-4xl mx-auto px-4">
                                {stats.map((stat, index) => (
                                    <div key={index} className="card-glass text-center p-6 hover-lift">
                                        <div className="text-3xl mb-2">{stat.icon}</div>
                                        <div className="text-2xl font-black gradient-text mb-1">{stat.number}</div>
                                        <div className="text-gray-600 text-sm font-semibold">{stat.label}</div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    </div>
                </section>

                {/* Features Section */}
                <section className="py-16">
                    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                        <div className="text-center mb-16">
                            <h2 className="text-3xl sm:text-4xl font-black text-gray-900 mb-6">
                                Everything you need in <span className="gradient-text">one place</span>
                            </h2>
                        </div>
                        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-8">
                            {features.map((feature, index) => (
                                <div key={index} className="group relative bg-white rounded-3xl p-8 shadow-lg hover:shadow-2xl transition-all duration-300 border border-gray-100">
                                    <div className={`w-16 h-16 bg-gradient-to-r ${feature.gradient} rounded-2xl flex items-center justify-center mb-6`}>
                                        <span className="text-3xl">{feature.icon}</span>
                                    </div>
                                    <h3 className="text-xl font-bold text-gray-900 mb-4">{feature.title}</h3>
                                    <p className="text-gray-600 leading-relaxed">{feature.description}</p>
                                </div>
                            ))}
                        </div>
                    </div>
                </section>

                {/* Testimonials */}
                <section className="py-16 bg-white/50 backdrop-blur-sm">
                    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                        <div className="text-center mb-12">
                            <h2 className="text-3xl font-black text-gray-900 mb-6">Loved by thousands</h2>
                        </div>
                        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
                            {testimonials.map((test, index) => (
                                <div key={index} className="card-glass p-8">
                                    <p className="text-gray-700 mb-6 italic">"{test.content}"</p>
                                    <div className="flex items-center space-x-3">
                                        <div className="w-10 h-10 bg-gradient-to-br from-purple-500 to-blue-500 rounded-full flex items-center justify-center text-white font-bold">
                                            {test.avatar}
                                        </div>
                                        <div>
                                            <div className="font-bold text-gray-900">{test.name}</div>
                                            <div className="text-sm text-gray-600">{test.role}</div>
                                        </div>
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>
                </section>

                <footer className="bg-gray-900 text-white py-12">
                    <div className="max-w-7xl mx-auto px-4 text-center">
                        <p className="text-gray-400">&copy; 2026 Payego. All rights reserved.</p>
                    </div>
                </footer>
            </div>
        </ErrorBoundary>
    );
};

export default LandingPage;
