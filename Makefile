.PHONY: run test lint fmt seed check

# Run the API
run:
	cargo run -p payego

# Run the seeder
seed:
	cargo run -p payego-seeder

# Run tests
test:
	cargo test --workspace

# Lint code
lint:
	cargo clippy --workspace --all-targets --all-features -- -D warnings
	cd payego_ui && npm run lint

# Format code
fmt:
	cargo fmt
	
# Check code
check:
	cargo check --workspace
