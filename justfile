# https://just.systems

@_:
   just --list

# Compile the Pony example projects
[group('build')]
pony-compile:
    ponyc tests/examples/pony_project1/src/
    ponyc tests/examples/pony_project2/src/

# Compile the Rust project
[group('build')]
compile:
    cargo build

[group('build')]
build:
    cargo build --release

# Remove build artifacts
[group('build')]
clean:
    rm -rf ./target

# Auto-fix formatting issues
[group('qa')]
fmt:
    cargo fmt --all

# Fix clippy issues automatically where possible
[group('qa')]
fix:
    cargo clippy --all-targets --all-features --fix --allow-dirty

# Install cargo-audit if not present
[group('setup')]
install-audit:
    cargo install cargo-audit --force

# Run tests with detailed output
[group('qa')]
test-basic:
    cargo test basic_actor_formatting

[group('qa')]
test-files:
    cargo test example_files_formatting -- --nocapture

# Check code formatting
[group('qa')]
fmt-check:
    cargo fmt --all -- --check

# Run clippy lints
[group('qa')]
clippy:
    cargo clippy --all-targets --all-features -- -D warnings

# Run security audit
[group('qa')]
audit:
    cargo audit

# Run all tests (replicates CI pipeline locally)
[group('qa')]
test:
    @echo "🔍 Checking code formatting..."
    cargo fmt --all -- --check
    @echo "📎 Running clippy lints..."
    cargo clippy --all-targets --all-features -- -D warnings
    @echo "🧪 Running unit tests..."
    cargo test --verbose
    @echo "🔒 Running security audit..."
    cargo audit
    @echo "🏗️  Building release..."
    cargo build --verbose --release
    @echo "✅ All local tests passed!"

# Quick test without security audit (for faster iteration)
[group('qa')]
test-fast:
    @echo "🔍 Checking code formatting..."
    cargo fmt --all -- --check
    @echo "📎 Running clippy lints..."
    cargo clippy --all-targets --all-features -- -D warnings
    @echo "🧪 Running unit tests..."
    cargo test --verbose
    @echo "✅ Fast tests passed!"

# Run only the failing test for debugging
[group('qa')]
test-debug:
    @echo "🐛 Running failing test with detailed output..."
    cargo test example_files_formatting -- --nocapture

# Simulate the exact CI environment locally
[group('qa')]
ci-local:
    @echo "🤖 Simulating CI environment locally..."
    @echo "📋 Rust version:"
    rustc --version
    @echo "📋 Cargo version:"
    cargo --version
    @echo ""
    just test

# Pre-commit hook simulation (fast feedback)
[group('qa')]
pre-commit:
    @echo "🪝 Running pre-commit checks..."
    just fmt-check
    just clippy
    just test-basic
    @echo "✅ Pre-commit checks passed!"
