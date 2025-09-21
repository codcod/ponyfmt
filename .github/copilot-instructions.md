# GitHub Copilot Instructions for PonyFmt

## Project Overview
This is a Rust-based code formatter for the Pony programming language. The project uses tree-sitter for parsing and implements custom formatting rules to ensure consistent code style across Pony codebases.

## Core Principles

### 1. Minimal Changes Philosophy
- **Make the smallest possible change** to achieve the desired outcome
- Preserve existing code structure and logic whenever possible
- Only modify what is strictly necessary for the specific task
- Avoid refactoring unless explicitly requested
- Keep existing variable names, function signatures, and module structure intact

### 2. Rust Best Practices

#### Code Quality
- Use `clippy::pedantic` and `clippy::nursery` lints as guidance
- Prefer `expect()` over `unwrap()` with descriptive error messages
- Use `?` operator for error propagation instead of explicit match statements
- Implement `Display` and `Error` traits for custom error types
- Use `#[must_use]` attribute for functions returning `Result` types

#### Memory Safety & Performance
- Use `Box<str>` instead of `String` for immutable string data
- Prefer `&str` over `String` for function parameters when ownership isn't needed
- Use `Cow<str>` for cases where you might need owned or borrowed strings
- Use `Vec::with_capacity()` when the final size is known or estimable
- Prefer iterators over index-based loops for better performance and safety

#### Error Handling
- Use `anyhow::Result` for application errors and `thiserror` for library errors
- Create specific error types for different failure modes in the formatter
- Always provide context when returning errors: `.context("operation description")`
- Use `bail!` macro from anyhow for early returns with error messages

#### Code Organization
- Keep modules focused and cohesive
- Use `pub(crate)` instead of `pub` for internal APIs
- Group related functionality in modules (e.g., `parser`, `formatter`, `debug`)
- Use explicit imports rather than glob imports (`use mod::*`)

### 3. Formatter-Specific Guidelines

#### Tree-Sitter Integration
- Handle all possible node types in the Pony grammar
- Use pattern matching exhaustively for AST node processing
- Implement graceful degradation for unknown or malformed nodes
- Cache tree-sitter queries when possible for performance

#### Formatting Rules
- Maintain consistency with existing Pony style conventions
- Preserve user's intent for complex expressions
- Handle edge cases gracefully (empty files, comments-only files, syntax errors)
- Respect existing blank lines and spacing where semantically meaningful
- Format comments appropriately while preserving their semantic positioning

#### Testing Strategy
- Write unit tests for individual formatting functions
- Include integration tests with real Pony code examples
- Test edge cases: empty files, syntax errors, deeply nested structures
- Use property-based testing where applicable (formatting should be idempotent)
- Test performance with large files

### 4. Dependencies and External Crates
- Minimize new dependencies - justify each addition
- Prefer well-maintained crates with good documentation
- Use `tree-sitter` and `tree-sitter-pony` for parsing
- Use `anyhow` for error handling in applications
- Use `clap` for CLI argument parsing with derive API
- Consider `rayon` for parallel processing of multiple files

### 5. Documentation Standards
- Document all public APIs with doc comments
- Include examples in doc comments for complex functions
- Explain the "why" not just the "what" in comments
- Document invariants and assumptions
- Keep README.md updated with usage examples and installation instructions

### 6. Performance Considerations
- Profile before optimizing - measure actual bottlenecks
- Use `criterion` for benchmarking performance-critical paths
- Consider streaming/incremental processing for large files
- Implement early returns for no-op formatting cases
- Use appropriate data structures (e.g., `SmallVec` for typically small collections)

## Code Style Preferences

### Variable Naming
- Use descriptive names: `node_kind` instead of `nk`
- Use `_result` suffix for unused Results that must be acknowledged
- Use `_guard` suffix for RAII guards
- Prefer full words over abbreviations

### Function Design
- Keep functions small and focused (< 50 lines typically)
- Use early returns to reduce nesting
- Separate parsing logic from formatting logic
- Make functions pure when possible (no side effects)

### Pattern Matching
- Use exhaustive matching for enums
- Prefer `if let` for single-variant matches
- Use `matches!` macro for boolean checks against patterns
- Group similar patterns in match arms

### Comments
- Use `//` for line comments, `///` for doc comments
- Explain complex algorithms or non-obvious business logic
- Add TODO comments with issue references for known technical debt
- Use SAFETY comments for unsafe code blocks

## Project-Specific Context

### Module Structure
- `main.rs`: CLI entry point and argument handling
- `lib.rs`: Public API and main formatter interface
- `parser.rs`: Tree-sitter integration and AST utilities
- `formatter.rs`: Core formatting logic and rules
- `debug.rs`: Debug utilities and AST visualization

### Key Data Structures
- Use consistent naming for AST-related types
- Implement `Debug` for all custom types
- Consider implementing `Clone` only when necessary
- Use newtype patterns for domain-specific string types

### Testing Patterns
- Place unit tests in the same file as the code they test
- Use `tests/` directory for integration tests
- Include example Pony files in `tests/examples/`
- Test both formatting and roundtrip properties (format → parse → format = same result)

## When Suggesting Changes
1. Always explain why the change improves the code
2. Consider backwards compatibility and API stability
3. Suggest the minimal change that solves the problem
4. Provide alternative approaches when multiple solutions exist
5. Consider the impact on performance and memory usage
6. Ensure changes align with Rust idioms and this project's goals

Remember: The goal is to create a reliable, fast, and maintainable Pony code formatter. Every suggestion should move us closer to that goal while following Rust best practices.
