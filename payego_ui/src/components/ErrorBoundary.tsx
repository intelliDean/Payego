import { Component, ReactNode } from "react";

interface Props {
    children: ReactNode;
}

interface State {
    hasError: boolean;
    error: Error | null;
}

class ErrorBoundary extends Component<Props, State> {
    state: State = { hasError: false, error: null };

    static getDerivedStateFromError(error: Error): State {
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
