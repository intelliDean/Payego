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
- All contributors to this project

## Built with love by: 
Michael Dean