//! # PonyFmt - Experimental Pony Code Formatter
//!
//! PonyFmt is a code formatter for the [Pony programming language](https://www.ponylang.io/)
//! written in Rust. It uses [tree-sitter](https://tree-sitter.github.io/tree-sitter/) with
//! the [tree-sitter-pony](https://github.com/tree-sitter-grammars/tree-sitter-pony) grammar
//! to parse Pony source code and format it according to consistent style rules.
//!
//! ## Status
//!
//! This is an early prototype. The formatter implements basic indentation and whitespace
//! normalization, but is not yet idempotent or specification-complete. It should be
//! considered experimental and used with caution on production code.
//!
//! ## Usage
//!
//! ### As a Library
//!
//! ```rust
//! use ponyfmt::formatter::{FormatOptions, Mode, format_source};
//!
//! let pony_source = r#"
//! actor Main
//! new create(env: Env) =>
//! env.out.print("Hello")
//! "#;
//!
//! let opts = FormatOptions {
//!     indent_width: 2,
//!     mode: Mode::Stdout,
//! };
//!
//! let formatted = format_source(pony_source, &opts).unwrap();
//! println!("{}", formatted);
//! ```
//!
//! ### As a CLI Tool
//!
//! The library is also available as a command-line tool. See the `main` module
//! for CLI usage details.
//!
//! ## Modules
//!
//! - [`parser`] - Tree-sitter integration and Pony source parsing
//! - [`formatter`] - Core formatting logic and public API
//!
//! ## Limitations
//!
//! - Formatting is not idempotent (running twice may produce different results)
//! - Limited comment preservation
//! - Basic error recovery for malformed source
//! - Performance not optimized for very large files

/// Tree-sitter based Pony language parser
pub mod parser;

/// Core formatting engine and public API
pub mod formatter;

#[cfg(test)]
mod debug;
