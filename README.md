# Payego 🚀

> **A modern, secure, and scalable payment processing platform built with Rust (Axum) and React.**

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-v1.83+-orange.svg)
![React](https://img.shields.io/badge/react-v19.1+-blue.svg)
![Status](https://img.shields.io/badge/status-active-success.svg)

## 📖 Overview

**Payego** is a comprehensive financial technology platform designed to simulate a modern payment system. It enables users to manage multi-currency wallets, process payments via major gateways (Stripe, PayPal, Paystack), and perform secure internal/external transfers.

Built with a philosophy of **"Safety First"**, Payego leverages Rust's memory safety and strong type system on the backend, ensuring distinct separation between Database Entities and API Contracts. The frontend provides a polished, responsive user experience.

---

## ✨ Key Features

### 🔐 Enterprise-Grade Security
-   **Secure Authentication**: JWT-based stateless auth with short-lived access tokens and refresh tokens.
-   **Configuration Security**: Sensitive keys (API secrets, DB passwords) are wrapped in `secrecy::Secret` types to prevent memory leaks and accidental logging.
-   **Rate Limiting**: Integrated `tower-governor` prevents API abuse and DDoS attacks.
-   **Type Safety**: API Data Transfer Objects (DTOs) are strictly separated from Database Entities, preventing data leakage (e.g., password hashes).

### 💰 Comprehensive Financial Tools
-   **Multi-Currency Wallets**: Real-time support for **USD**, **EUR**, **GBP**, and **NGN**.
-   **Global Payments**:
    -   **Stripe**: Secure credit card processing.
    -   **PayPal**: International secure checkout.
    -   **Paystack**: African market integration with NGAN support.
-   **Bank Integration**: Verify bank accounts and process direct withdrawals.

### ⚡ Performance & Observability
-   **Structured Logging**: Production-ready JSON logs with unique `X-Request-ID` tracing for every request.
-   **Async Core**: Built on `Tokio` and `Axum` for massive concurrency support.
-   **Optimized Database**: PostgreSQL with `Diesel` ORM connection pooling (`r2d2`).

---

## 🏗️ Architecture

### Backend (`/src`)
The backend follows a **Clean Architecture** pattern:
-   **Handlers** (`src/handlers`): Thin layer responsible only for HTTP request parsing and response formatting.
-   **Services** (`src/services`): Contain all business logic (e.g., `TransferService`, `PaymentService`, `AuthService`). This layer is unit-testable.
-   **Models** (`src/models`):
    -   `entities.rs`: Direct mappings to PostgreSQL tables.
    -   `dtos.rs`: User-facing API structures (Input/Output).
    -   `app_state.rs`: Thread-safe application state container.

### Frontend (`/payego_ui`)
Modern React application built with **Vite**:
-   **Components**: Modular, reusable UI elements tailored with TailwindCSS.
-   **API Client**: Centralized `Axios` instance with automatic auth header injection and global error handling.
-   **Testing**: Infrastructure set up with **Vitest** and **React Testing Library**.

---

## 🚀 Getting Started

### Prerequisites
-   **Rust**: v1.75+ (`rustup update`)
-   **Node.js**: v18+
-   **PostgreSQL**: v15+
-   **Docker** (Optional)

### 1. Backend Setup

1.  **Clone & Enter:**
    ```bash
    git clone https://github.com/intelliDean/payego.git
    cd payego
    ```

2.  **Environment Configuration:**
    Create a `.env` file in the root directory:
    ```env
    DATABASE_URL=postgres://user:password@localhost/payego
    JWT_SECRET=super_secret_key_must_be_32_chars_long
    JWT_EXPIRATION_HOURS=2
    STRIPE_SECRET_KEY=sk_test_...
    PAYPAL_CLIENT_ID=...
    PAYPAL_SECRET=...
    PAYSTACK_SECRET_KEY=sk_test_...
    APP_URL=http://localhost:8080
    CORS_ORIGINS=http://localhost:5173
    RUST_LOG=info
    ```

3.  **Database Setup:**
    ```bash
    cargo install diesel_cli --no-default-features --features postgres
    diesel setup
    diesel migration run
    ```

4.  **Run Server:**
    ```bash
    cargo run
    ```
    Server starts at `http://127.0.0.1:8080`.

### 2. Frontend Setup

1.  **Navigate & Install:**
    ```bash
    cd payego_ui
    npm install
    ```

2.  **Run Development Server:**
    ```bash
    npm run dev
    ```
    UI available at `http://localhost:5173`.

---

## 📚 API Documentation

Payego includes auto-generated **Swagger/OpenAPI** documentation.
Once the server is running, visit:
👉 **[http://localhost:8080/swagger-ui/](http://localhost:8080/swagger-ui/)**

---

## 🧪 Testing

### Backend tests
Run unit tests for Services and Integration tests for Handlers:
```bash
cargo test
```

### Frontend tests
Run Vitest for the React application:
```bash
cd payego_ui
npm test
```

---

## 🐳 Docker Deployment

To spin up the entire stack (Postgres + Backend + Frontend is WIP) or just backend dependencies:

```bash
docker-compose up -d
```

This creates a Postgres container pre-configured for the application.

---

## 🤝 Contributing

1.  Fork the repo.
2.  Create a feature branch (`git checkout -b feature/NewThing`).
3.  Commit changes (`git commit -m 'Add NewThing'`).
4.  Push to branch (`git push origin feature/NewThing`).
5.  Open a Pull Request.

---

##  Built with ❤️ By

**Michael Dean Oyewole**

---

## 📝 License

This project is licensed under the MIT License.



