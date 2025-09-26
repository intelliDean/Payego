// import React, { useState } from 'react';
// import axios from 'axios';
//
// function LoginForm() {
//     const [email, setEmail] = useState('');
//     const [password, setPassword] = useState('');
//     const [error, setError] = useState(null);
//     const [loading, setLoading] = useState(false);
//
//     const handleLogin = async (e) => {
//         e.preventDefault();
//         setLoading(true);
//         setError(null);
//         try {
//
//             const response = await axios.post(
//                 `${import.meta.env.VITE_API_URL}/api/login`,
//                 { email, password },
//                 {
//                     headers: { 'Content-Type': 'application/json' },
//                 }
//             );
//             console.log("response: ", response);
//
//
//             localStorage.setItem('jwt_token', response.data.token);
//             window.location.href = '/'; // Redirect to top-up form
//         } catch (err) {
//             setError(err.response?.data?.message || 'Login failed');
//         } finally {
//             setLoading(false);
//         }
//     };
//
//     return (
//         <div style={{ maxWidth: '400px', margin: '0 auto', padding: '20px' }}>
//             <h2>Login</h2>
//             <form onSubmit={handleLogin}>
//                 <div>
//                     <label>
//                         Email:
//                         <input
//                             type="email"
//                             value={email}
//                             onChange={(e) => setEmail(e.target.value)}
//                             style={{ margin: '10px', width: '100%' }}
//                             required
//                         />
//                     </label>
//                 </div>
//                 <div>
//                     <label>
//                         Password:
//                         <input
//                             type="password"
//                             value={password}
//                             onChange={(e) => setPassword(e.target.value)}
//                             style={{ margin: '10px', width: '100%' }}
//                             required
//                         />
//                     </label>
//                 </div>
//                 <button
//                     type="submit"
//                     disabled={loading}
//                     style={{ margin: '10px', padding: '10px', width: '100%' }}
//                 >
//                     {loading ? 'Logging in...' : 'Login'}
//                 </button>
//                 {error && <p style={{ color: 'red' }}>{error}</p>}
//             </form>
//         </div>
//     );
// }
//
// export default LoginForm;



import React, { useState } from 'react';
import axios from 'axios';
import { Link } from 'react-router-dom';

function LoginForm({ setAuth }) {
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
                { headers: { 'Content-Type': 'application/json' } }
            );
            localStorage.setItem('jwt_token', response.data.token);
            setAuth(true);
        } catch (err) {
            setError(err.response?.data?.message || 'Login failed');
        } finally {
            setLoading(false);
        }
    };

    return (
        <div className="max-w-md mx-auto mt-10 p-6 bg-white rounded-lg shadow-md">
            <h2 className="text-2xl font-bold mb-6 text-center">Login</h2>
            <form onSubmit={handleLogin}>
                <div className="mb-4">
                    <label className="block text-gray-700">Email</label>
                    <input
                        type="email"
                        value={email}
                        onChange={(e) => setEmail(e.target.value)}
                        className="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                        required
                    />
                </div>
                <div className="mb-4">
                    <label className="block text-gray-700">Password</label>
                    <input
                        type="password"
                        value={password}
                        onChange={(e) => setPassword(e.target.value)}
                        className="w-full p-2 border rounded focus:outline-none focus:ring-2 focus:ring-blue-500"
                        required
                    />
                </div>
                <button
                    type="submit"
                    disabled={loading}
                    className="w-full bg-blue-600 text-white p-2 rounded hover:bg-blue-700 disabled:bg-blue-300"
                >
                    {loading ? 'Logging in...' : 'Login'}
                </button>
                {error && <p className="text-red-500 mt-4 text-center">{error}</p>}
            </form>
            <p className="mt-4 text-center">
                Don't have an account? <Link to="/register" className="text-blue-600 hover:underline">Register</Link>
            </p>
        </div>
    );
}

export default LoginForm;