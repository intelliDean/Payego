# Contributing to Payego

Thank you for your interest in contributing to Payego! We welcome contributions from everyone and appreciate your help in making this project better.

## 📋 Table of Contents

- [Code of Conduct](#code-of-conduct)
- [Getting Started](#getting-started)
- [Development Workflow](#development-workflow)
- [Testing Requirements](#testing-requirements)
- [Code Quality Standards](#code-quality-standards)
- [Pull Request Process](#pull-request-process)
- [Commit Message Guidelines](#commit-message-guidelines)
- [Architecture Overview](#architecture-overview)
- [Security Guidelines](#security-guidelines)
- [Documentation Standards](#documentation-standards)
- [Getting Help](#getting-help)

---

## 🤝 Code of Conduct

### Our Pledge

We are committed to providing a welcoming and inclusive environment for all contributors, regardless of experience level, background, or identity.

### Expected Behavior

- Be respectful and considerate
- Use welcoming and inclusive language
- Accept constructive criticism gracefully
- Focus on what's best for the project
- Show empathy towards other contributors

### Unacceptable Behavior

- Harassment, trolling, or discriminatory comments
- Personal attacks or insults
- Publishing others' private information
- Any conduct that would be inappropriate in a professional setting

---

## 🚀 Getting Started

### Prerequisites

Ensure you have the following installed:
- **Rust**: v1.75+ (`rustup update`)
- **Node.js**: v18+
- **PostgreSQL**: v15+
- **Diesel CLI**: `cargo install diesel_cli --no-default-features --features postgres`
- **Git**: Latest version

### Initial Setup

1. **Fork the Repository**
   ```bash
   # Click "Fork" on GitHub, then clone your fork
   git clone https://github.com/YOUR_USERNAME/payego.git
   cd payego
   ```

2. **Add Upstream Remote**
   ```bash
   git remote add upstream https://github.com/intelliDean/payego.git
   ```

3. **Environment Configuration**
   ```bash
   cp .env.example .env
   ```
   
   Edit `.env` and configure:
   - Database credentials
   - JWT secret (32+ characters)
   - Payment provider keys (Stripe, PayPal, Paystack)
   - CORS origins

4. **Database Setup**
   ```bash
   # Create database
   diesel setup
   
   # Run migrations
   diesel migration run
   ```

5. **Install Dependencies**
   
   **Backend:**
   ```bash
   cargo build
   ```
   
   **Frontend:**
   ```bash
   cd payego_ui
   npm install
   ```

6. **Verify Setup**
   
   **Backend:**
   ```bash
   cargo test --workspace
   cargo run
   ```
   
   **Frontend:**
   ```bash
   cd payego_ui
   npm test
   npm run dev
   ```

---

## 🔄 Development Workflow

### 1. Create a Feature Branch

Always create a new branch for your work:
```bash
git checkout -b feature/your-feature-name
# or
git checkout -b fix/bug-description
```

**Branch naming conventions:**
- `feature/` - New features
- `fix/` - Bug fixes
- `docs/` - Documentation changes
- `refactor/` - Code refactoring
- `test/` - Adding or updating tests
- `chore/` - Maintenance tasks

### 2. Make Your Changes

**Backend Development:**
- Code location: `crates/api`, `crates/core`, `crates/primitives`
- Run tests: `cargo test --workspace`
- Check linting: `cargo clippy --workspace --tests -- -D warnings`
- Format code: `cargo fmt --all`

**Frontend Development:**
- Code location: `payego_ui/src`
- Run dev server: `npm run dev`
- Run tests: `npm test`
- Run linting: `npm run lint`
- Build: `npm run build`

### 3. Keep Your Branch Updated

Regularly sync with upstream:
```bash
git fetch upstream
git rebase upstream/main
```

### 4. Commit Your Changes

Follow our [commit message guidelines](#commit-message-guidelines):
```bash
git add .
git commit -m "feat: add user profile endpoint"
```

### 5. Push and Create PR

```bash
git push origin feature/your-feature-name
```

Then create a Pull Request on GitHub.

---

## 🧪 Testing Requirements

### Backend Tests

**Required:**
- Write integration tests for new API endpoints
- Write unit tests for service layer logic
- Ensure all existing tests pass

**Running tests:**
```bash
# All tests
cargo test --workspace

# Specific test file
cargo test --test auth_tests

# With output
cargo test -- --nocapture
```

**Test categories:**
- `bin/payego/tests/` - Integration tests
- Service unit tests - In service modules

**Example test:**
```rust
#[tokio::test]
async fn test_user_registration() {
    let app = setup_test_app().await;
    let response = app.register_user("test@example.com", "password123").await;
    assert_eq!(response.status(), StatusCode::CREATED);
}
```

### Frontend Tests

**Required:**
- Write tests for new components
- Test error handling scenarios
- Ensure all existing tests pass

**Running tests:**
```bash
cd payego_ui
npm test

# With coverage
npm test -- --coverage

# Watch mode
npm test -- --watch
```

**Test utilities:**
Use `test-utils.tsx` for component tests:
```typescript
import { render, screen } from '../utils/test-utils';

test('renders login form', () => {
  render(<LoginForm />);
  expect(screen.getByLabelText(/email/i)).toBeInTheDocument();
});
```

### Test Coverage Expectations

- **Backend**: Maintain 70%+ coverage for service layer
- **Frontend**: Aim for 60%+ coverage for components
- **Critical paths**: 100% coverage (auth, payments, transfers)

---

## ✅ Code Quality Standards

### Rust Code Quality

**1. Run Clippy (Required)**
```bash
cargo clippy --workspace --tests -- -D warnings
```

All Clippy warnings must be resolved before PR approval.

**2. Format Code (Required)**
```bash
cargo fmt --all
```

**3. Follow Rust Conventions**
- Use `snake_case` for functions and variables
- Use `PascalCase` for types and traits
- Add doc comments for public APIs
- Avoid `unwrap()` - use proper error handling

**4. Error Handling**
- Use `Result<T, ApiError>` for all fallible operations
- Never use `panic!` in production code
- Log errors appropriately (WARN for expected, ERROR for unexpected)

**5. Security**
- Wrap secrets in `secrecy::Secret<T>`
- Never log sensitive data
- Validate all user input

### Frontend Code Quality

**1. Run Linter (Required)**
```bash
npm run lint
```

**2. TypeScript**
- Enable strict mode
- Avoid `any` types
- Use proper type definitions

**3. Component Standards**
- Use functional components with hooks
- Extract reusable logic into custom hooks
- Keep components focused and small

**4. Error Handling**
- Use centralized error handler (`errorHandler.ts`)
- Display user-friendly error messages
- Log errors for debugging

---

## 📝 Pull Request Process

### Before Submitting

**Checklist:**
- [ ] All tests pass (`cargo test`, `npm test`)
- [ ] Code is formatted (`cargo fmt`, `npm run lint`)
- [ ] No Clippy warnings (`cargo clippy`)
- [ ] Added tests for new functionality
- [ ] Updated documentation if needed
- [ ] Commit messages follow conventions
- [ ] Branch is up-to-date with `main`

### PR Description Template

```markdown
## Description
Brief description of changes

## Type of Change
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

## Testing
How was this tested?

## Checklist
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
```

### Review Process

1. **Automated Checks**: CI runs tests and linting
2. **Code Review**: Maintainer reviews code
3. **Feedback**: Address review comments
4. **Approval**: PR approved by maintainer
5. **Merge**: Squash and merge to main

**Expected timeline:** 2-5 business days for initial review

---

## 💬 Commit Message Guidelines

We follow [Conventional Commits](https://www.conventionalcommits.org/).

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation changes
- `style`: Code style changes (formatting)
- `refactor`: Code refactoring
- `test`: Adding or updating tests
- `chore`: Maintenance tasks

### Examples

**Good:**
```
feat(auth): add email verification flow

Implements email verification with expiring tokens.
Users receive verification emails upon registration.

Closes #123
```

**Bad:**
```
updated stuff
```

### Rules

- Use present tense ("add" not "added")
- Use imperative mood ("move" not "moves")
- Keep subject line under 72 characters
- Reference issues in footer

---

## 🏗️ Architecture Overview

### Crate Structure

```
payego/
├── bin/payego/          # Main binary
├── crates/
│   ├── api/             # HTTP handlers
│   ├── core/            # Business logic
│   └── primitives/      # Shared types
└── payego_ui/           # React frontend
```

### Backend Layers

**1. API Layer** (`crates/api`)
- HTTP handlers
- Request/response parsing
- Route definitions
- OpenAPI documentation

**2. Core Layer** (`crates/core`)
- Business logic services
- External API clients
- Database operations
- Email service

**3. Primitives Layer** (`crates/primitives`)
- Database entities
- DTOs (Data Transfer Objects)
- Error types
- Shared utilities

### Key Patterns

**Service Layer Pattern:**
```rust
pub struct TransferService {
    db: Arc<DatabaseConnection>,
}

impl TransferService {
    pub async fn transfer_internal(
        &self,
        request: TransferRequest,
    ) -> Result<Transaction, ApiError> {
        // Business logic here
    }
}
```

**Entity vs DTO Separation:**
- **Entities**: Database models (internal only)
- **DTOs**: API contracts (public-facing)
- Never expose entities directly in API responses

**Error Handling:**
```rust
pub enum ApiError {
    NotFound(String),
    Unauthorized,
    ValidationError(Vec<String>),
    // ...
}
```

### Frontend Architecture

**Component Structure:**
- Keep components small and focused
- Extract business logic into hooks
- Use React Query for server state
- Use Context API for auth state

**Error Handling:**
```typescript
import { getErrorMessage } from '../utils/errorHandler';

try {
  await api.login(credentials);
} catch (err) {
  setError(getErrorMessage(err));
}
```

---

## 🔒 Security Guidelines

### Handling Secrets

**DO:**
- Use `secrecy::Secret<String>` for sensitive data
- Store secrets in environment variables
- Use `.env` file locally (never commit!)

**DON'T:**
- Hardcode secrets in code
- Log sensitive information
- Commit `.env` files

### Reporting Vulnerabilities

**If you discover a security vulnerability:**
1. **DO NOT** open a public issue
2. Email the maintainer directly
3. Include detailed description and reproduction steps
4. Allow time for fix before public disclosure

### Security Best Practices

- Validate all user input
- Use parameterized queries (Diesel handles this)
- Implement rate limiting
- Use HTTPS in production
- Keep dependencies updated

---

## 📚 Documentation Standards

### Code Comments

**Rust:**
```rust
/// Transfers funds between internal Payego users
///
/// # Arguments
/// * `from_user_id` - Source user ID
/// * `to_username` - Destination username
/// * `amount` - Transfer amount
///
/// # Returns
/// Transaction record on success
///
/// # Errors
/// Returns `ApiError::InsufficientBalance` if sender lacks funds
pub async fn transfer_internal(
    &self,
    from_user_id: Uuid,
    to_username: &str,
    amount: Decimal,
) -> Result<Transaction, ApiError> {
    // Implementation
}
```

**TypeScript:**
```typescript
/**
 * Extracts user-friendly error message from API error
 * @param error - Axios error or generic error object
 * @returns User-friendly error message string
 */
export function getErrorMessage(error: any): string {
  // Implementation
}
```

### API Documentation

**OpenAPI/Swagger:**
- Document all endpoints with `#[utoipa::path]`
- Include request/response examples
- Document all status codes
- Add parameter descriptions

**Example:**
```rust
#[utoipa::path(
    post,
    path = "/api/auth/login",
    request_body = LoginRequest,
    responses(
        (status = 200, description = "Login successful", body = AuthResponse),
        (status = 401, description = "Invalid credentials"),
    ),
    tag = "Authentication"
)]
pub async fn login(/* ... */) -> Result<Json<AuthResponse>, ApiError> {
    // Implementation
}
```

### README Updates

If your changes affect:
- Setup process → Update README.md
- Docker deployment → Update README.Docker.md
- Contributing process → Update CONTRIBUTING.md

---

## 🆘 Getting Help

### Resources

- **Main README**: [README.md](README.md)
- **Docker Guide**: [README.Docker.md](README.Docker.md)
- **API Docs**: http://localhost:8080/swagger-ui/ (when running)

### Questions?

- Open a GitHub Discussion for general questions
- Open an issue for bug reports
- Check existing issues before creating new ones

### Development Tips

**Debugging Backend:**
```bash
# Enable debug logging
RUST_LOG=debug cargo run

# View database queries
RUST_LOG=diesel=debug cargo run
```

**Debugging Frontend:**
```bash
# Check React Query cache
# Install React Query DevTools (already configured)
```

**Database Inspection:**
```bash
# Connect to database
diesel database reset  # Careful: drops all data!
psql -U postgres -d payego
```

---

## 🎉 Thank You!

Your contributions make Payego better for everyone. We appreciate your time and effort!

**Happy Coding!** 🚀

---

## 📄 License

By contributing to Payego, you agree that your contributions will be licensed under the MIT License.
