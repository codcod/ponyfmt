//! Pony language parser using tree-sitter
//!
//! This module provides a simple interface to parse Pony source code using the
//! [tree-sitter-pony](https://github.com/tree-sitter-grammars/tree-sitter-pony) grammar.
//! It wraps the tree-sitter parsing functionality and provides the parsed AST
//! for use by the formatter.
//!
//! # Example
//!
//! ```rust
//! use ponyfmt::parser::parse;
//!
//! let source = r#"
//! actor Main
//!   new create(env: Env) =>
//!     env.out.print("Hello, World!")
//! "#;
//!
//! let tree = parse(source).unwrap();
//! let root = tree.root_node();
//! println!("Parsed {} node with {} children", root.kind(), root.child_count());
//! ```

use once_cell::sync::Lazy;
use tree_sitter::{Language, Parser, Tree};
use tree_sitter_pony::language as pony_language;

/// The Pony language definition for tree-sitter
///
/// This static instance is initialized lazily and provides the Pony grammar
/// for tree-sitter parsing operations.
pub static PONY_LANGUAGE: Lazy<Language> = Lazy::new(pony_language);

/// Parse Pony source code into an AST
///
/// This function creates a new tree-sitter parser configured for the Pony language
/// and parses the provided source code string.
///
/// # Arguments
///
/// * `source` - The Pony source code to parse
///
/// # Returns
///
/// Returns a `Result` containing the parsed `Tree` on success, or an error if
/// parsing fails. Common causes of failure include:
/// - Invalid or severely malformed syntax
/// - Internal tree-sitter parsing errors
///
/// # Example
///
/// ```rust
/// use ponyfmt::parser::parse;
///
/// let source = "actor Main\n  new create(env: Env) => None";
/// match parse(source) {
///     Ok(tree) => println!("Successfully parsed {} nodes", tree.root_node().child_count()),
///     Err(e) => eprintln!("Parse error: {}", e),
/// }
/// ```
///
/// # Note
///
/// The parser is somewhat tolerant of syntax errors and will attempt to produce
/// a partial AST even for malformed input. Check the tree for error nodes if
/// you need to validate syntax correctness.
pub fn parse(source: &str) -> anyhow::Result<Tree> {
    let mut parser = Parser::new();
    parser.set_language(*PONY_LANGUAGE)?;
    parser
        .parse(source, None)
        .ok_or_else(|| anyhow::anyhow!("Failed to parse Pony source"))
}
