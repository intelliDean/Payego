import React, { Component } from "react";

class ErrorBoundary extends Component {
    state = { hasError: false, error: null };

    static getDerivedStateFromError(error) {
        return { hasError: true, error };
    }

    render() {
        if (this.state.hasError) {
            return (
                <div className="max-w-4xl mx-auto mt-10 p-8 bg-white rounded-2xl shadow-xl border border-gray-100">
                    <h2 className="text-2xl font-bold text-red-600 mb-4">Oops, something broke!</h2>
                    <p className="text-gray-600 mb-4">Looks like our app took a coffee break. Try refreshing the page!</p>
                    <p className="text-sm text-gray-500">Error: {this.state.error?.message || "Unknown error"}</p>
                </div>
            );
        }
        return this.props.children;
    }
}

export default ErrorBoundary;
