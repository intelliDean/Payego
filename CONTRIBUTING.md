# Contributing to Payego

Thank you for your interest in contributing to Payego! We welcome contributions from everyone.

## Development Workflow

1.  **Fork and Clone**: Fork the repository and clone it locally.
2.  **Branching**: Create a new branch for your feature or bugfix.
    ```bash
    git checkout -b feature/amazing-feature
    ```
3.  **Backend Development**:
    -   Code is in `src/`, `crates/`, and `bin/`.
    -   Run tests: `cargo test`
    -   Check linting: `cargo clippy`
    -   Check formatting: `cargo fmt`
4.  **Frontend Development**:
    -   Code is in `payego_ui/`.
    -   Run dev server: `npm run dev`
    -   Run tests: `npm run test`
    -   Run linting: `npm run lint`

## Pull Request Process

1.  Ensure all local tests pass.
2.  Commit your changes with clear, descriptive commit messages.
3.  Push to your fork and submit a Pull Request to the `main` branch.
4.  Our CI pipeline will automatically run to verify your changes.

## Code Style

-   **Rust**: Follow standard Rust conventions (`rustfmt`).
-   **JavaScript/React**: Follow ESLint configuration in `payego_ui`.

## Architecture Guide

-   **payego-api** (`crates/api`): HTTP handlers and routing.
-   **payego-core** (`crates/core`): Business logic and services.
-   **payego-primitives** (`crates/primitives`): Shared types, DTOs, and errors.
-   **payego-ui** (`payego_ui`): React frontend.

Happy Coding! 🚀
