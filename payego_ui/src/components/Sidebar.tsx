import React from 'react';
import { Link, useLocation } from 'react-router-dom';
import { useAuth } from '../contexts/AuthContext';
import ThemeToggle from './ThemeToggle';
import { useTheme } from '../contexts/ThemeContext';

interface SidebarProps {
    isOpen: boolean;
    setIsOpen: (isOpen: boolean) => void;
}

const Sidebar: React.FC<SidebarProps> = ({ isOpen, setIsOpen }) => {
    const { logout } = useAuth();
    const location = useLocation();

    const menuItems = [
        { to: "/dashboard", label: "Dashboard", icon: "🏠" },
        { to: "/top-up", label: "Top Up", icon: "💰" },
        { to: "/transfer", label: "Transfer", icon: "💸" },
        { to: "/withdraw", label: "Withdraw", icon: "🏦" },
        { to: "/convert", label: "Convert", icon: "🔄" },
        { to: "/banks", label: "Banks", icon: "🏛️" },
        { to: "/wallets", label: "Wallets", icon: "💳" },
        { to: "/transactions", label: "Transactions", icon: "📜" },
        { to: "/profile", label: "Profile", icon: "👤" },
    ];

    return (
        <>
            {/* Mobile Overlay */}
            {isOpen && (
                <div
                    className="fixed inset-0 bg-black/20 backdrop-blur-sm z-40 md:hidden"
                    onClick={() => setIsOpen(false)}
                />
            )}

            <aside className={`fixed inset-y-0 left-0 w-64 bg-white dark:bg-slate-900 border-r border-gray-200 dark:border-slate-800 shadow-2xl transform ${isOpen ? "translate-x-0" : "-translate-x-full"} transition-transform duration-300 ease-in-out z-50 overflow-y-auto`}>
                <div className="p-6">
                    <div className="flex items-center justify-between mb-8">
                        <Link to="/dashboard" className="flex items-center space-x-3 group" onClick={() => setIsOpen(false)}>
                            <div className="w-10 h-10 bg-gradient-to-br from-purple-600 to-blue-600 rounded-xl flex items-center justify-center shadow-lg group-hover:scale-105 transition-transform">
                                <span className="text-white font-bold text-lg">P</span>
                            </div>
                            <h2 className="text-2xl font-black gradient-text">Payego</h2>
                        </Link>
                        <div className="flex items-center space-x-2">
                            <ThemeToggle />
                            <button
                                onClick={() => setIsOpen(false)}
                                className="md:hidden p-2 text-gray-400 hover:text-gray-600 transition-colors"
                            >
                                ✕
                            </button>
                        </div>
                    </div>

                    <nav className="space-y-1.5">
                        {menuItems.map((item) => {
                            const isActive = location.pathname === item.to;
                            return (
                                <Link
                                    key={item.to}
                                    to={item.to}
                                    className={`flex items-center space-x-3 p-3 rounded-xl transition-all duration-200 group ${isActive
                                        ? "bg-gradient-to-r from-blue-600/10 to-indigo-600/10 text-blue-700 shadow-sm"
                                        : "text-gray-600 hover:bg-gray-50 hover:text-gray-900"
                                        }`}
                                    onClick={() => setIsOpen(false)}
                                >
                                    <span className={`text-xl transition-transform duration-200 ${isActive ? "scale-110" : "group-hover:scale-110"}`}>
                                        {item.icon}
                                    </span>
                                    <span className="font-bold">{item.label}</span>
                                    {isActive && (
                                        <div className="ml-auto w-1.5 h-1.5 rounded-full bg-blue-600 shadow-glow" />
                                    )}
                                </Link>
                            );
                        })}

                        <div className="pt-4 mt-4 border-t border-gray-100">
                            <button
                                onClick={logout}
                                className="flex items-center space-x-3 p-3 rounded-xl text-gray-600 hover:bg-red-50 hover:text-red-700 transition-all duration-200 w-full text-left font-bold group"
                            >
                                <span className="text-xl group-hover:scale-110 transition-transform">🚪</span>
                                <span>Log Out</span>
                            </button>
                        </div>
                    </nav>

                    {/* Pro Badge Placeholder */}
                    <div className="mt-10 p-4 bg-gradient-to-br from-gray-900 to-blue-900 rounded-2xl text-white shadow-xl relative overflow-hidden group">
                        <div className="absolute top-0 right-0 w-24 h-24 bg-white/5 rounded-full -mr-12 -mt-12 group-hover:scale-150 transition-transform duration-500"></div>
                        <p className="text-xs font-bold text-blue-400 uppercase tracking-widest mb-1">Upgrade To Pro</p>
                        <p className="text-sm font-medium text-gray-300 mb-3">Get unlimited wallets & priority support.</p>
                        <button className="w-full py-2 bg-blue-600 rounded-lg text-xs font-black uppercase hover:bg-blue-500 transition-colors">
                            Learn More
                        </button>
                    </div>
                </div>
            </aside>
        </>
    );
};

export default Sidebar;
