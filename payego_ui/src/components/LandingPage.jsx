import React, { useEffect, useState } from 'react';
import { useNavigate } from 'react-router-dom';
import ErrorBoundary from './ErrorBoundary';

function LandingPage() {
    const navigate = useNavigate();
    const [isVisible, setIsVisible] = useState(false);

    useEffect(() => {
        setIsVisible(true);
    }, []);

    const handleGetStarted = () => {
        navigate('/register');
    };

    const handleLogin = () => {
        navigate('/login');
    };

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
                            <div className="flex justify-center mb-8 sm:mb-10">
                                <div className="relative group">
                                    <div className="absolute inset-0 bg-gradient-to-r from-purple-600 to-blue-600 rounded-3xl blur-lg opacity-50 group-hover:opacity-75 transition-opacity duration-300"></div>
                                    <div className="relative w-20 sm:w-24 lg:w-28 h-20 sm:h-24 lg:h-28 bg-gradient-to-br from-purple-600 via-blue-600 to-indigo-600 rounded-3xl flex items-center justify-center shadow-2xl transform group-hover:scale-105 transition-transform duration-300">
                                        <span className="text-white font-black text-3xl sm:text-4xl lg:text-5xl">P</span>
                                    </div>
                                    <div className="absolute -top-2 -right-2 w-8 sm:w-10 h-8 sm:h-10 bg-gradient-to-r from-green-400 to-emerald-500 rounded-full flex items-center justify-center shadow-lg animate-bounce-subtle">
                                        <span className="text-white text-sm sm:text-base font-bold">✓</span>
                                    </div>
                                </div>
                            </div>

                            {/* Headline */}
                            <h1 className="text-4xl sm:text-5xl md:text-6xl lg:text-7xl xl:text-8xl font-black text-gray-900 mb-6 sm:mb-8 leading-tight px-4">
                                Your Money,{' '}
                                <span className="relative inline-block">
                                    <span className="gradient-text">Simplified</span>
                                    <svg className="absolute -bottom-2 left-0 w-full" height="12" viewBox="0 0 200 12" fill="none" xmlns="http://www.w3.org/2000/svg">
                                        <path d="M2 10C50 2 150 2 198 10" stroke="url(#gradient)" strokeWidth="3" strokeLinecap="round" />
                                        <defs>
                                            <linearGradient id="gradient" x1="0%" y1="0%" x2="100%" y2="0%">
                                                <stop offset="0%" stopColor="#8b5cf6" />
                                                <stop offset="100%" stopColor="#3b82f6" />
                                            </linearGradient>
                                        </defs>
                                    </svg>
                                </span>
                            </h1>

                            {/* Subheadline */}
                            <p className="text-lg sm:text-xl md:text-2xl lg:text-3xl text-gray-600 mb-8 sm:mb-10 max-w-4xl mx-auto leading-relaxed px-4 font-medium">
                                The modern digital wallet that makes managing multiple currencies as easy as sending a text.
                                <span className="block mt-2 gradient-text font-semibold">Top up, withdraw, transfer, and convert with confidence.</span>
                            </p>

                            {/* CTA Buttons */}
                            <div className="flex flex-col sm:flex-row gap-4 sm:gap-5 justify-center items-center mb-12 sm:mb-16 px-4">
                                <button
                                    onClick={handleGetStarted}
                                    className="w-full sm:w-auto btn-primary-glow btn-lg group"
                                >
                                    <span className="relative z-10 flex items-center justify-center space-x-2">
                                        <span>Get Started Free</span>
                                        <svg className="w-5 h-5 transform group-hover:translate-x-1 transition-transform" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M13 7l5 5m0 0l-5 5m5-5H6" />
                                        </svg>
                                    </span>
                                </button>

                                <button
                                    onClick={handleLogin}
                                    className="w-full sm:w-auto btn-secondary btn-lg group"
                                >
                                    <span className="flex items-center justify-center space-x-2">
                                        <svg className="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M11 16l-4-4m0 0l4-4m-4 4h14m-5 4v1a3 3 0 01-3 3H6a3 3 0 01-3-3V7a3 3 0 013-3h7a3 3 0 013 3v1" />
                                        </svg>
                                        <span>Sign In</span>
                                    </span>
                                </button>
                            </div>

                            {/* Trust Indicators */}
                            <div className="flex flex-wrap justify-center items-center gap-6 sm:gap-8 mb-12 px-4 text-sm text-gray-600">
                                <div className="flex items-center space-x-2">
                                    <svg className="w-5 h-5 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                                        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                                    </svg>
                                    <span className="font-semibold">Bank-level security</span>
                                </div>
                                <div className="flex items-center space-x-2">
                                    <svg className="w-5 h-5 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                                        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                                    </svg>
                                    <span className="font-semibold">No hidden fees</span>
                                </div>
                                <div className="flex items-center space-x-2">
                                    <svg className="w-5 h-5 text-green-500" fill="currentColor" viewBox="0 0 20 20">
                                        <path fillRule="evenodd" d="M10 18a8 8 0 100-16 8 8 0 000 16zm3.707-9.293a1 1 0 00-1.414-1.414L9 10.586 7.707 9.293a1 1 0 00-1.414 1.414l2 2a1 1 0 001.414 0l4-4z" clipRule="evenodd" />
                                    </svg>
                                    <span className="font-semibold">24/7 support</span>
                                </div>
                            </div>

                            {/* Stats */}
                            <div className="grid grid-cols-2 md:grid-cols-4 gap-6 sm:gap-8 max-w-4xl mx-auto px-4">
                                {stats.map((stat, index) => (
                                    <div
                                        key={index}
                                        className="card-glass text-center p-6 hover-lift"
                                        style={{ animationDelay: `${index * 100}ms` }}
                                    >
                                        <div className="text-3xl mb-2">{stat.icon}</div>
                                        <div className="text-2xl sm:text-3xl lg:text-4xl font-black gradient-text mb-1">{stat.number}</div>
                                        <div className="text-gray-600 text-sm sm:text-base font-semibold">{stat.label}</div>
                                    </div>
                                ))}
                            </div>
                        </div>
                    </div>
                </section>

                {/* Features Section */}
                <section className="py-16 sm:py-20 lg:py-24 relative">
                    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                        <div className="text-center mb-16 sm:mb-20">
                            <div className="inline-block mb-4 px-4 py-2 bg-purple-100 text-purple-700 rounded-full text-sm font-bold">
                                FEATURES
                            </div>
                            <h2 className="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-black text-gray-900 mb-6 px-4">
                                Everything you need in{' '}
                                <span className="gradient-text">one place</span>
                            </h2>
                            <p className="text-lg sm:text-xl lg:text-2xl text-gray-600 max-w-3xl mx-auto px-4">
                                Powerful features designed to make your financial life simpler and more secure
                            </p>
                        </div>

                        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-6 sm:gap-8">
                            {features.map((feature, index) => (
                                <div
                                    key={index}
                                    className="group relative bg-white rounded-3xl p-8 shadow-lg hover:shadow-2xl transform hover:-translate-y-2 transition-all duration-300 border border-gray-100 overflow-hidden"
                                    style={{ animationDelay: feature.delay }}
                                >
                                    {/* Gradient overlay on hover */}
                                    <div className="absolute inset-0 bg-gradient-to-br from-purple-50 via-blue-50 to-transparent opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>

                                    <div className="relative z-10">
                                        <div className={`w-16 h-16 bg-gradient-to-r ${feature.gradient} rounded-2xl flex items-center justify-center mb-6 group-hover:scale-110 transition-transform duration-300 shadow-lg`}>
                                            <span className="text-3xl">{feature.icon}</span>
                                        </div>

                                        <h3 className="text-xl sm:text-2xl font-bold text-gray-900 mb-4">
                                            {feature.title}
                                        </h3>

                                        <p className="text-base text-gray-600 leading-relaxed">
                                            {feature.description}
                                        </p>
                                    </div>

                                    {/* Decorative corner element */}
                                    <div className="absolute -bottom-4 -right-4 w-24 h-24 bg-gradient-to-br from-purple-100 to-blue-100 rounded-full opacity-0 group-hover:opacity-50 transition-opacity duration-300"></div>
                                </div>
                            ))}
                        </div>
                    </div>
                </section>

                {/* Testimonials Section */}
                <section className="py-16 sm:py-20 lg:py-24 bg-white/50 backdrop-blur-sm">
                    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                        <div className="text-center mb-16">
                            <div className="inline-block mb-4 px-4 py-2 bg-blue-100 text-blue-700 rounded-full text-sm font-bold">
                                TESTIMONIALS
                            </div>
                            <h2 className="text-3xl sm:text-4xl md:text-5xl font-black text-gray-900 mb-6">
                                Loved by <span className="gradient-text">thousands</span>
                            </h2>
                            <p className="text-lg sm:text-xl text-gray-600 max-w-2xl mx-auto">
                                See what our users have to say about their experience
                            </p>
                        </div>

                        <div className="grid grid-cols-1 md:grid-cols-3 gap-8">
                            {testimonials.map((testimonial, index) => (
                                <div key={index} className="card-glass p-8 hover-lift">
                                    <div className="flex items-center mb-4">
                                        {[...Array(testimonial.rating)].map((_, i) => (
                                            <svg key={i} className="w-5 h-5 text-yellow-400 fill-current" viewBox="0 0 20 20">
                                                <path d="M10 15l-5.878 3.09 1.123-6.545L.489 6.91l6.572-.955L10 0l2.939 5.955 6.572.955-4.756 4.635 1.123 6.545z" />
                                            </svg>
                                        ))}
                                    </div>
                                    <p className="text-gray-700 mb-6 italic leading-relaxed">"{testimonial.content}"</p>
                                    <div className="flex items-center space-x-3">
                                        <div className="w-12 h-12 bg-gradient-to-br from-purple-500 to-blue-500 rounded-full flex items-center justify-center text-white font-bold">
                                            {testimonial.avatar}
                                        </div>
                                        <div>
                                            <div className="font-bold text-gray-900">{testimonial.name}</div>
                                            <div className="text-sm text-gray-600">{testimonial.role}</div>
                                        </div>
                                    </div>
                                </div>
                            ))}
                        </div>
                    </div>
                </section>

                {/* CTA Section */}
                <section className="py-16 sm:py-20 lg:py-24 relative overflow-hidden">
                    <div className="absolute inset-0 bg-gradient-to-r from-purple-600 via-blue-600 to-indigo-600"></div>
                    <div className="absolute inset-0 bg-black/10"></div>

                    {/* Animated circles */}
                    <div className="absolute top-0 left-0 w-96 h-96 bg-white/10 rounded-full filter blur-3xl animate-pulse-slow"></div>
                    <div className="absolute bottom-0 right-0 w-96 h-96 bg-white/10 rounded-full filter blur-3xl animate-pulse-slow" style={{ animationDelay: '1s' }}></div>

                    <div className="relative max-w-5xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
                        <h2 className="text-3xl sm:text-4xl md:text-5xl lg:text-6xl font-black text-white mb-6 px-4">
                            Ready to take control of your finances?
                        </h2>

                        <p className="text-lg sm:text-xl lg:text-2xl text-blue-100 mb-10 max-w-3xl mx-auto px-4 leading-relaxed">
                            Join thousands of users who trust Payego with their financial needs.
                            <span className="block mt-2 font-semibold">Get started in less than 2 minutes.</span>
                        </p>

                        <div className="flex flex-col sm:flex-row gap-4 sm:gap-5 justify-center items-center px-4">
                            <button
                                onClick={handleGetStarted}
                                className="w-full sm:w-auto px-8 py-4 bg-white text-purple-600 rounded-2xl font-bold text-lg shadow-2xl hover:shadow-3xl transform hover:-translate-y-1 transition-all duration-300 hover:bg-gray-50"
                            >
                                Create Free Account
                            </button>

                            <button
                                onClick={handleLogin}
                                className="w-full sm:w-auto px-8 py-4 bg-transparent text-white rounded-2xl font-bold text-lg border-2 border-white/30 hover:border-white/50 hover:bg-white/10 backdrop-blur-sm transform hover:-translate-y-1 transition-all duration-300"
                            >
                                Sign In Instead
                            </button>
                        </div>
                    </div>
                </section>

                {/* Footer */}
                <footer className="bg-gray-900 text-white py-12 sm:py-16">
                    <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
                        <div className="grid grid-cols-1 md:grid-cols-4 gap-8 mb-12">
                            {/* Brand */}
                            <div className="col-span-1 md:col-span-2">
                                <div className="flex items-center space-x-3 mb-4">
                                    <div className="w-10 h-10 bg-gradient-to-r from-purple-600 to-blue-600 rounded-xl flex items-center justify-center">
                                        <span className="text-white font-bold text-lg">P</span>
                                    </div>
                                    <span className="text-2xl font-black">Payego</span>
                                </div>
                                <p className="text-gray-400 max-w-md leading-relaxed">
                                    The modern digital wallet that makes managing multiple currencies as easy as sending a text.
                                </p>
                            </div>

                            {/* Links */}
                            <div>
                                <h4 className="font-bold mb-4">Product</h4>
                                <ul className="space-y-2 text-gray-400">
                                    <li><a href="#" className="hover:text-white transition-colors">Features</a></li>
                                    <li><a href="#" className="hover:text-white transition-colors">Pricing</a></li>
                                    <li><a href="#" className="hover:text-white transition-colors">Security</a></li>
                                </ul>
                            </div>

                            <div>
                                <h4 className="font-bold mb-4">Company</h4>
                                <ul className="space-y-2 text-gray-400">
                                    <li><a href="#" className="hover:text-white transition-colors">About</a></li>
                                    <li><a href="#" className="hover:text-white transition-colors">Privacy</a></li>
                                    <li><a href="#" className="hover:text-white transition-colors">Terms</a></li>
                                    <li><a href="#" className="hover:text-white transition-colors">Support</a></li>
                                </ul>
                            </div>
                        </div>

                        <div className="border-t border-gray-800 pt-8 text-center text-gray-400">
                            <p>&copy; 2025 Payego. All rights reserved. Built with ❤️ for the modern world.</p>
                        </div>
                    </div>
                </footer>
            </div>
        </ErrorBoundary>
    );
}

export default LandingPage;