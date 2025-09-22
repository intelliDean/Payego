import React, { useState } from 'react';
import axios from 'axios';

function LoginForm() {
    const [email, setEmail] = useState('');
    const [password, setPassword] = useState('');
    const [error, setError] = useState(null);
    const [loading, setLoading] = useState(false);

    const handleLogin = async (e) => {
        e.preventDefault();
        setLoading(true);
        setError(null);
        try {

            const response = await axios.post(
                `${import.meta.env.VITE_API_URL}/api/login`,
                { email, password },
                {
                    headers: { 'Content-Type': 'application/json' },
                }
            );
            console.log("response: ", response);


            localStorage.setItem('jwt_token', response.data.token);
            window.location.href = '/'; // Redirect to top-up form
        } catch (err) {
            setError(err.response?.data?.message || 'Login failed');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div style={{ maxWidth: '400px', margin: '0 auto', padding: '20px' }}>
            <h2>Login</h2>
            <form onSubmit={handleLogin}>
                <div>
                    <label>
                        Email:
                        <input
                            type="email"
                            value={email}
                            onChange={(e) => setEmail(e.target.value)}
                            style={{ margin: '10px', width: '100%' }}
                            required
                        />
                    </label>
                </div>
                <div>
                    <label>
                        Password:
                        <input
                            type="password"
                            value={password}
                            onChange={(e) => setPassword(e.target.value)}
                            style={{ margin: '10px', width: '100%' }}
                            required
                        />
                    </label>
                </div>
                <button
                    type="submit"
                    disabled={loading}
                    style={{ margin: '10px', padding: '10px', width: '100%' }}
                >
                    {loading ? 'Logging in...' : 'Login'}
                </button>
                {error && <p style={{ color: 'red' }}>{error}</p>}
            </form>
        </div>
    );
}

export default LoginForm;