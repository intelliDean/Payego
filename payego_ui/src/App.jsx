import React from 'react';
import { BrowserRouter, Routes, Route } from 'react-router-dom';
import TopUpForm from './components/TopUpForm';
import LoginForm from './components/LoginForm';

function App() {
    return (
        <BrowserRouter>
            <Routes>
                <Route path="/login" element={<LoginForm />} />
                <Route path="/" element={<TopUpForm />} />
                <Route path="/success" element={<div>Payment Successful!</div>} />
                <Route path="/cancel" element={<div>Payment Cancelled</div>} />
            </Routes>
        </BrowserRouter>
    );
}

export default App;