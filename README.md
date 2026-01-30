# Payego #RustAfricaHackathon

> **A modern, secure, and scalable payment processing platform built with Rust (Axum) and React.**

![License](https://img.shields.io/badge/license-MIT-blue.svg)
![Rust](https://img.shields.io/badge/rust-v1.83+-orange.svg)
![React](https://img.shields.io/badge/react-v19.1+-blue.svg)
![Status](https://img.shields.io/badge/status-active-success.svg)
![Tests](https://img.shields.io/badge/tests-31%20passing-success.svg)

## 📖 Overview

**Payego** is a comprehensive financial technology platform designed to simulate a modern payment system. It enables users to manage multi-currency wallets, process payments via major gateways (Stripe, PayPal, Paystack), and perform secure internal/external transfers.

Built with a philosophy of **"Safety First"**, Payego leverages Rust's memory safety and strong type system on the backend, ensuring distinct separation between Database Entities and API Contracts. The frontend provides a polished, responsive user experience with centralized error handling #RustAfricaHackathon

---

## ✨ Key Features

### 🔐 Enterprise-Grade Security
-   **Secure Authentication**: JWT-based stateless auth with short-lived access tokens and refresh tokens
-   **Email Verification**: Secure email verification flow with expiring tokens and resend capability
-   **Configuration Security**: Sensitive keys (API secrets, DB passwords) wrapped in `secrecy::Secret` types to prevent memory leaks and accidental logging
-   **Rate Limiting**: Integrated `tower-governor` prevents API abuse and DDoS attacks
-   **Type Safety**: API Data Transfer Objects (DTOs) strictly separated from Database Entities, preventing data leakage (e.g., password hashes)
-   **Audit Logging**: Comprehensive audit trail for all user actions (login, registration, transfers, etc.)

### 💰 Comprehensive Financial Tools
-   **Multi-Currency Wallets**: Real-time support for **20+ currencies** including USD, EUR, GBP, NGN, CAD, AUD, CHF, JPY, CNY, and more
-   **Currency Conversion**: Internal currency conversion with real-time exchange rates and 1% fee
-   **Global Payments**:
    -   **Stripe**: Secure credit card processing with webhook support
    -   **PayPal**: International secure checkout
    -   **Paystack**: African market integration with NGN support and account verification
-   **Bank Integration**: 
    -   Verify Nigerian bank accounts via Paystack
    -   Process direct withdrawals to bank accounts
    -   Add/remove bank accounts with confirmation flow
-   **Transfer Types**:
    -   **Internal**: Transfer between Payego users by username
    -   **External**: Transfer to external bank accounts
    -   **Idempotency**: Duplicate request prevention with idempotency keys

### ⚡ Performance & Observability
-   **Structured Logging**: Production-ready JSON logs with unique `X-Request-ID` tracing for every request
-   **Async Core**: Built on `Tokio` and `Axum` for massive concurrency support
-   **Optimized Database**: PostgreSQL with `Diesel` ORM connection pooling (`r2d2`)
-   **Comprehensive Testing**: 31 passing backend integration tests covering all major flows

---

## 🏗️ Architecture

### Backend Structure

The backend follows a **Clean Architecture** pattern with three main crates:

```
payego/
├── bin/payego/              # Main application binary
│   └── tests/               # Integration tests (31 tests)
├── crates/
│   ├── api/                 # HTTP handlers & routing (payego-api)
│   │   └── src/handlers/    # API endpoint handlers
│   ├── core/                # Business logic (payego-core)
│   │   ├── src/services/    # Service layer (auth, payment, transfer, etc.)
│   │   └── src/clients/     # External API clients (Stripe, PayPal, Paystack)
│   └── primitives/          # Shared types (payego-primitives)
│       └── src/models/      # Database entities & DTOs
└── payego_ui/               # React frontend
    └── src/
        ├── components/      # React components
        ├── utils/           # Utilities (error handling, etc.)
        └── api/             # API client
```

```mermaid
graph TD
    Client[Client / Frontend] -->|HTTP / JSON| API[Payego API Crate]
    
    subgraph "Payego Backend"
        API -->|Calls| Core[Payego Core Crate]
        Core -->|Uses| Primitives[Payego Primitives Crate]
        API -->|Uses| Primitives
        
        Core -->|Business Logic| Services[Services]
        Services -->|Database Access| Models[Models / Diesel]
    end
    
    Services -->|External API| Stripe[Stripe]
    Services -->|External API| PayPal[PayPal]
    Services -->|External API| Paystack[Paystack]
    Services -->|SMTP| Email[Email Service]
    
    Models -->|SQL| DB[(PostgreSQL)]
```

**Key Components:**
-   **API Crate** (`crates/api`): Thin HTTP layer with handlers for routing and request/response formatting
-   **Core Crate** (`crates/core`): Business logic services (AuthService, TransferService, PaymentService, etc.)
-   **Primitives Crate** (`crates/primitives`): Shared types, database entities, and DTOs

### Frontend (`/payego_ui`)

Modern React application built with **Vite**:
-   **Components**: Modular, reusable UI elements with TailwindCSS
-   **Centralized Error Handling**: Custom error utility extracts user-friendly messages from API responses
-   **API Client**: Axios instance with automatic auth header injection and 401 redirect
-   **State Management**: React Query for server state, Context API for auth
-   **Testing**: Vitest and React Testing Library infrastructure

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
    Copy `.env.example` to `.env` and configure:
    ```env
    # Database
    DATABASE_URL=postgres://user:password@localhost/payego
    
    # JWT Configuration
    JWT_SECRET=super_secret_key_must_be_32_chars_long
    JWT_EXPIRATION_HOURS=2
    REFRESH_TOKEN_EXPIRATION_DAYS=7
    
    # Payment Providers
    STRIPE_SECRET_KEY=sk_test_...
    STRIPE_WEBHOOK_SECRET=whsec_...
    PAYPAL_CLIENT_ID=...
    PAYPAL_SECRET=...
    PAYSTACK_SECRET_KEY=sk_test_...
    PAYSTACK_PUBLIC_KEY=pk_test_...
    
    # Application URLs
    APP_URL=http://localhost:8080
    FRONTEND_URL=http://localhost:5173
    CORS_ORIGINS=http://localhost:5173
    
    # Email (Optional - uses mock in development)
    SMTP_HOST=smtp.gmail.com
    SMTP_PORT=587
    SMTP_USERNAME=your-email@gmail.com
    SMTP_PASSWORD=your-app-password
    SMTP_FROM=noreply@payego.com
    
    # Logging
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

Payego includes auto-generated **Swagger/OpenAPI** documentation with complete endpoint descriptions, request/response schemas, and status codes.

Once the server is running, visit:
👉 **[http://localhost:8080/swagger-ui/](http://localhost:8080/swagger-ui/)**

### Quick API Reference

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/auth/register` | POST | Register new user |
| `/api/auth/login` | POST | Login with credentials |
| `/api/auth/refresh` | POST | Refresh access token |
| `/api/auth/verify-email` | POST | Verify email with token |
| `/api/auth/resend-verification` | POST | Resend verification email |
| `/api/user/me` | GET | Get current user info |
| `/api/wallet/top-up` | POST | Initiate payment (Stripe/PayPal) |
| `/api/wallet/transfer/internal` | POST | Transfer to Payego user |
| `/api/wallet/transfer/external` | POST | Transfer to bank account |
| `/api/wallet/convert` | POST | Convert between currencies |
| `/api/wallet/withdraw/:id` | POST | Withdraw to bank account |
| `/api/bank/add` | POST | Add bank account |
| `/api/bank/delete/:id` | DELETE | Remove bank account |
| `/api/bank/resolve` | GET | Verify bank account details |
| `/api/bank/user` | GET | List user's bank accounts |
| `/api/transactions/:id` | GET | Get transaction details |
| `/api/transactions/user` | GET | Get user transaction history |
| `/api/audit-logs` | GET | Get user audit logs |

---

## 🧪 Testing

### Backend Tests (31 passing)

Run comprehensive integration tests:
```bash
cargo test --workspace
```

**Test Coverage:**
- ✅ Authentication & JWT (7 tests)
- ✅ Email Verification (2 tests)
- ✅ Audit Logging (2 tests)
- ✅ Banking Operations (4 tests)
- ✅ Currency Conversion (3 tests)
- ✅ Payments (1 test)
- ✅ Transactions (4 tests)
- ✅ Withdrawals (3 tests)
- ✅ Wallets (2 tests)
- ✅ Idempotency (multiple tests)
- ✅ Rate Limiting
- ✅ Error Handling

### Frontend Tests

Run Vitest for the React application:
```bash
cd payego_ui
npm test
```

**Current Coverage:**
- ✅ Error handler utility tests
- 🚧 Component tests (in progress)

### Code Quality

Check code quality with Clippy:
```bash
cargo clippy --workspace --tests -- -D warnings
```

Format code:
```bash
cargo fmt --all
```

---

## 🐳 Docker Deployment

The simplest way to run Payego in production or development is using Docker Compose.

1.  **Configure environment**: Copy `.env.example` to `.env` and fill in your secrets.
2.  **Launch stack**:
    ```bash
    docker-compose up -d --build
    ```

This command builds the optimized production image, starts the application, initializes the PostgreSQL database, and automatically runs all migrations.

-   **Backend**: `http://localhost:8081` (Internal 8080)
-   **Frontend**: Build and serve with nginx
-   **Database**: `localhost:5434` (Internal 5432)
-   **Swagger UI**: `http://localhost:8081/swagger-ui/`

---

## 🔑 Key Technical Decisions

### Idempotency
All financial operations (transfers, withdrawals, conversions) require an `idempotency_key` to prevent duplicate transactions. The system tracks processed keys and returns the original result for duplicate requests.

### Email Verification
- New users receive verification emails with 24-hour expiring tokens
- Mock email client in development (logs to console)
- SMTP configuration optional for production

### Error Handling
- Backend: Structured error types with proper HTTP status codes
- Frontend: Centralized error handler extracts user-friendly messages
- All errors logged with request IDs for debugging

### Audit Logging
All sensitive operations (login, registration, transfers) are logged to the `audit_logs` table with:
- User ID
- Action type
- IP address
- Timestamp
- Additional metadata

---

## 🤝 Contributing

1.  Fork the repo
2.  Create a feature branch (`git checkout -b feature/NewThing`)
3.  Commit changes (`git commit -m 'Add NewThing'`)
4.  Push to branch (`git push origin feature/NewThing`)
5.  Open a Pull Request

**Development Guidelines:**
- Write tests for new features
- Run `cargo clippy` before committing
- Follow existing code style
- Update documentation as needed

---

## 🛠️ Tech Stack

**Backend:**
- Rust 1.83+
- Axum (web framework)
- Tokio (async runtime)
- Diesel (ORM)
- PostgreSQL (database)
- Tower (middleware)
- Utoipa (OpenAPI docs)

**Frontend:**
- React 19.1+
- TypeScript
- Vite (build tool)
- TailwindCSS (styling)
- React Query (data fetching)
- React Hook Form + Zod (forms & validation)
- Axios (HTTP client)

**External Services:**
- Stripe (payments)
- PayPal (payments)
- Paystack (African payments & bank verification)

---

##  Built for #RustAfricaHackathon By

**Michael Dean Oyewole**

---

## 📝 License

This project is licensed under the MIT License.
