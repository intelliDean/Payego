# Payego

A modern, secure, and scalable payment processing platform built with Rust and React.

## Overview

Payego is a comprehensive financial platform that enables users to manage multiple currencies, process payments through various payment gateways, and handle both domestic and international transfers. Built with security and performance in mind, it leverages Rust's safety guarantees on the backend and React's component-based architecture on the frontend.

## Features

- 🔐 **Secure Authentication**
  - JWT-based authentication
  - Password hashing with bcrypt
  - Session management

- 💰 **Multi-Currency Support**
  - Support for 20 major currencies
  - Real-time currency conversion
  - Separate wallet for each currency

- 🏦 **Bank Integration**
  - Bank account verification
  - Multiple bank account management
  - Support for various banking systems

- 💳 **Payment Gateway Integration**
  - Stripe payment processing
  - PayPal integration
  - Paystack support (for Nigerian banks)

- 📱 **User-Friendly Interface**
  - Modern React-based UI
  - Responsive design with TailwindCSS
  - Intuitive transaction management

- 🔄 **Transaction Management**
  - Internal transfers between users
  - External bank transfers
  - Detailed transaction history
  - Multiple payment methods

## Technology Stack

### Backend

- **Framework**: Axum (Rust)
- **Database**: PostgreSQL with Diesel ORM
- **Authentication**: JWT
- **API Documentation**: OpenAPI/Swagger
- **Payment Processing**: Stripe, PayPal, Paystack

### Frontend

- **Framework**: React 19
- **Build Tool**: Vite
- **Styling**: TailwindCSS
- **State Management**: React Hooks
- **Routing**: React Router

### DevOps

- **Containerization**: Docker
- **Database Migration**: Diesel CLI
- **Multi-architecture Support**: amd64/arm64

## Prerequisites

- Rust (Latest stable version)
- Node.js (v18 or higher)
- PostgreSQL (v15 or higher)
- Docker (optional, for containerized deployment)

## Getting Started

### Backend Setup

1. **Clone the repository**
   ```bash
   git clone git@github.com:intelliDean/payego.git
   cd payego
   ```

2. **Set up the database**
   ```bash
   diesel setup
   diesel migration run
   ```

3. **Configure environment variables**
   Create a .env file in the root directory with the following variables:
   ```env
   DATABASE_URL=postgresql://user:password@localhost/payego
   JWT_SECRET=your_jwt_secret
   STRIPE_SECRET_KEY=your_stripe_secret
   PAYPAL_CLIENT_ID=your_paypal_client_id
   PAYPAL_SECRET=your_paypal_secret
   PAYSTACK_SECRET_KEY=your_paystack_secret
   ```

4. **Run the backend**
   ```bash
   cargo run
   ```

### Frontend Setup

1. **Navigate to the frontend directory**
   ```bash
   cd payego_ui
   ```

2. **Install dependencies**
   ```bash
   npm install
   ```

3. **Start the development server**
   ```bash
   npm run dev
   ```

### Docker Deployment

To run the entire application using Docker:

```bash
docker compose up --build
```

The application will be available at `localhost:8080`

## API Documentation

Once the backend is running, you can access the Swagger UI documentation at:
`localhost:8080/swagger-ui/`

## Database Schema

The application uses the following main tables:

- `users`: User account information
- `wallets`: Multi-currency wallet management
- `transactions`: Transaction history and status
- `bank_accounts`: User bank account details
- `banks`: Supported banks information

## Security Features

- Password hashing using bcrypt
- JWT-based authentication
- Input validation and sanitization
- CORS protection
- Rate limiting
- Secure headers

## Testing

### Test Payment Cards

For testing Stripe integration, use Stripe's official test cards:

**Successful Payment**:
- Card Number: `4242 4242 4242 4242`
- Expiry: Any future date (e.g., `12/34`)
- CVC: Any 3 digits (e.g., `123`)
- ZIP: Any 5 digits (e.g., `12345`)

**Declined Payment**:
- Card Number: `4000 0000 0000 0002`

**Requires Authentication (3D Secure)**:
- Card Number: `4000 0025 0000 3155`

**Insufficient Funds**:
- Card Number: `4000 0000 0000 9995`

For a complete list of test cards, see: [Stripe Testing Documentation](https://stripe.com/docs/testing#cards)

### Test PayPal Account

Use PayPal Sandbox accounts for testing:
1. Create test accounts at: [PayPal Developer Dashboard](https://developer.paypal.com/dashboard/accounts)
2. Use sandbox credentials in your `.env` file
3. Test transactions will appear in the sandbox dashboard

### Test Paystack

Use Paystack test mode with test secret key:
- Test cards: [Paystack Test Payments](https://paystack.com/docs/payments/test-payments)
- Successful test card: `5060 6666 6666 6666 666` (Verve)
- Declined test card: `5060 0000 0000 0000 000`

### Running Tests

```bash
# Run backend tests
cargo test

# Run frontend tests
cd payego_ui
npm test
```

## Contributing

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support, please open an issue in the GitHub repository.

## Acknowledgments

- Rust Community
- React Team
- All future contributors to this project

## Built with love by: 
Michael Dean