//! Core formatting engine for Pony source code
//!
//! This module contains the main formatting logic that transforms parsed Pony AST
//! nodes into formatted source code. The formatter implements indentation rules,
//! spacing conventions, and other style guidelines for Pony code.
//!
//! # Example
//!
//! ```rust
//! use ponyfmt::formatter::{FormatOptions, Mode, format_source};
//!
//! let unformatted = r#"
//! actor Main
//! new create(env:Env)=>
//! env.out.print("test")
//! "#;
//!
//! let opts = FormatOptions {
//!     indent_width: 2,
//!     mode: Mode::Stdout,
//! };
//!
//! let formatted = format_source(unformatted, &opts).unwrap();
//! // Result will have proper indentation and spacing
//! ```

use crate::parser::parse;
use anyhow::Result;
use tree_sitter::{Node, TreeCursor};

/// Output mode for the formatter
///
/// Determines how the formatted code should be handled after processing.
#[derive(Clone, Copy, Debug)]
pub enum Mode {
    /// Print formatted code to stdout
    Stdout,
    /// Write formatted code back to source files
    Write,
    /// Check if formatting would change the code (used for CI/validation)
    Check,
}

/// Configuration options for the formatter
///
/// Controls various aspects of the formatting behavior including indentation
/// and output mode.
///
/// # Example
///
/// ```rust
/// use ponyfmt::formatter::{FormatOptions, Mode};
///
/// let opts = FormatOptions {
///     indent_width: 4,  // Use 4 spaces for indentation
///     mode: Mode::Write,  // Write changes back to files
/// };
/// ```
pub struct FormatOptions {
    /// Number of spaces to use for each indentation level
    pub indent_width: usize,
    /// How to handle the formatted output
    pub mode: Mode,
}

impl Default for FormatOptions {
    fn default() -> Self {
        Self {
            indent_width: 2,
            mode: Mode::Stdout,
        }
    }
}

#[derive(Debug)]
struct FormatterState {
    output: String,
    indent_level: usize,
    last_byte: usize,
    needs_newline: bool,
    needs_space: bool,
    in_conditional_context: bool,
    conditional_base_indent: Option<usize>,
}

impl FormatterState {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            last_byte: 0,
            needs_newline: false,
            needs_space: false,
            in_conditional_context: false,
            conditional_base_indent: None,
        }
    }

    fn write_indent(&mut self, opts: &FormatOptions) {
        if self.needs_newline {
            self.output.push('\n');
            self.output
                .push_str(&" ".repeat(self.indent_level * opts.indent_width));
            self.needs_newline = false;
            self.needs_space = false;
        } else if self.needs_space {
            self.output.push(' ');
            self.needs_space = false;
        }
    }

    fn write_token(&mut self, text: &str, opts: &FormatOptions) {
        self.write_indent(opts);
        self.output.push_str(text);

        // Add space after certain tokens
        match text {
            "=" | "," | "|" | ";" => {
                self.request_space();
            }
            // Add space after keywords that are typically followed by other tokens
            "use" | "trait" | "class" | "actor" | "primitive" | "new" | "fun" | "let" | "var"
            | "is" => {
                self.request_space();
            }
            _ => {}
        }
    }

    fn request_newline(&mut self) {
        self.needs_newline = true;
    }

    fn request_space(&mut self) {
        if !self.needs_newline {
            self.needs_space = true;
        }
    }
}

/// Format Pony source code according to style conventions
///
/// This is the main entry point for the formatter. It parses the input source code
/// into an AST using tree-sitter and then walks the tree to generate formatted output
/// according to the provided options.
///
/// # Arguments
///
/// * `input` - The Pony source code to format
/// * `opts` - Configuration options controlling formatting behavior
///
/// # Returns
///
/// Returns a `Result` containing the formatted source code as a `String`, or an error
/// if parsing or formatting fails.
///
/// # Errors
///
/// This function will return an error if:
/// - The input source code cannot be parsed by tree-sitter
/// - Internal formatting logic encounters an unrecoverable error
///
/// # Example
///
/// ```rust
/// use ponyfmt::formatter::{FormatOptions, Mode, format_source};
///
/// let source = r#"
/// actor Main
/// new create(env:Env)=>
/// env.out.print("Hello")
/// "#;
///
/// let opts = FormatOptions {
///     indent_width: 2,
///     mode: Mode::Stdout,
/// };
///
/// let formatted = format_source(source, &opts).unwrap();
/// assert!(formatted.contains("  new create(env: Env) =>"));
/// ```
///
/// # Current Limitations
///
/// - Formatting is not idempotent (multiple runs may produce different results)
/// - Comment positioning may not be preserved perfectly
/// - Some complex expressions may not format optimally
/// - Error recovery for malformed source is limited
pub fn format_source(input: &str, opts: &FormatOptions) -> Result<String> {
    let tree = parse(input)?;
    let mut state = FormatterState::new();
    let mut cursor = tree.walk();

    format_node(&mut cursor, input.as_bytes(), &mut state, opts);

    // Add trailing newline for proper file formatting
    if !state.output.is_empty() && !state.output.ends_with('\n') {
        state.output.push('\n');
    }

    Ok(state.output)
}

fn format_node(
    cursor: &mut TreeCursor,
    source: &[u8],
    state: &mut FormatterState,
    opts: &FormatOptions,
) {
    format_node_impl(cursor, source, state, opts, true);
}

fn handle_intervening_content(
    source: &[u8],
    state: &mut FormatterState,
    start: usize,
    end: usize,
    _opts: &FormatOptions,
) {
    if start < end && start < source.len() {
        let slice = &source[start..end.min(source.len())];
        if let Ok(text) = std::str::from_utf8(slice) {
            // Check for significant whitespace that should be preserved
            let has_newlines = text.contains('\n');
            let is_just_whitespace = text.trim().is_empty();

            if !is_just_whitespace {
                // There's non-whitespace content (like comments) - preserve it
                state.output.push_str(text.trim_start());
            }

            // If there were newlines and we have content after, preserve some spacing
            if has_newlines && !state.output.is_empty() {
                let needs_separation =
                    !state.output.ends_with('\n') && !state.output.ends_with(' ');
                if needs_separation {
                    state.request_newline();
                }
            }
        }
    }
    state.last_byte = end;
}

fn format_node_impl(
    cursor: &mut TreeCursor,
    source: &[u8],
    state: &mut FormatterState,
    opts: &FormatOptions,
    is_first_sibling: bool,
) {
    let node = cursor.node();

    // Handle whitespace and comments between last position and this node
    handle_intervening_content(source, state, state.last_byte, node.start_byte(), opts);

    if node.child_count() == 0 {
        // Leaf node - write the token
        if let Ok(text) = node.utf8_text(source) {
            handle_token_spacing(node, state, is_first_sibling);
            state.write_token(text, opts);

            // Handle post-token formatting
            let node_kind = node.kind();
            if node_kind == "block_comment" || node_kind == "line_comment" {
                // Add double newline after comments for proper separation
                state.request_newline();
                state.request_newline();
            }
        }
    } else {
        // Non-leaf node - handle formatting rules
        handle_node_formatting(cursor, source, state, opts);
    }

    state.last_byte = node.end_byte();
}

fn handle_node_formatting(
    cursor: &mut TreeCursor,
    source: &[u8],
    state: &mut FormatterState,
    opts: &FormatOptions,
) {
    let node = cursor.node();
    let node_kind = node.kind();

    // Handle top-level constructs - add newlines between them
    if matches!(
        node_kind,
        "actor_definition"
            | "class_definition"
            | "trait_definition"
            | "primitive_definition"
            | "struct_definition"
            | "interface_definition"
            | "object_definition"
            | "use_statement"
            | "type_alias"
    ) {
        // Reset indent level for top-level constructs
        state.indent_level = 0;

        if !state.output.is_empty() {
            // Ensure we have at least one newline before top-level declarations
            if !state.output.ends_with('\n') {
                state.request_newline();
            }
            // Add an extra newline for proper separation (except after comments)
            if !state.output.trim_end().ends_with("*/") {
                state.request_newline();
            }
        }
    }

    // Handle newlines and indentation for various constructs
    match node_kind {
        "method" | "constructor" | "behavior" | "function" => {
            state.request_newline();
            state.indent_level = 1; // Methods are indented one level inside classes/actors
        }
        "class_definition" | "actor_definition" | "trait_definition" => {
            // Class/actor body content should be indented
            state.indent_level = 0; // Reset for the class declaration itself
        }
        _ => {}
    }

    // Handle comment blocks - ensure newline after
    if node_kind == "comment" {
        // Comments will be handled in the intervening content, but ensure newline after
    }

    // Handle arguments - force single line formatting for simple function calls
    if node_kind == "arguments" {
        // Always try single line formatting for now
        handle_single_line_arguments(cursor, source, state, opts);
        return;
    }

    if cursor.goto_first_child() {
        let mut first_child = true;
        let mut found_arrow = false;
        let mut in_class_body = false;

        // Track if we're entering a class/actor/trait body
        if matches!(
            node_kind,
            "class_definition" | "actor_definition" | "trait_definition"
        ) {
            in_class_body = true;
        }

        loop {
            let child_node = cursor.node();
            let child_kind = child_node.kind();

            // Handle spacing before child nodes
            if !first_child {
                handle_child_spacing(child_node, state);
            }

            // Special indentation handling for class body elements
            if in_class_body
                && matches!(
                    child_kind,
                    "let_declaration"
                        | "var_declaration"
                        | "method"
                        | "constructor"
                        | "behavior"
                        | "function"
                )
            {
                if !first_child {
                    state.request_newline();
                }
                state.indent_level = 1; // Indent class members
            }

            // Special handling for conditional context in ERROR nodes only
            if state.in_conditional_context
                && child_kind == "identifier"
                && state.output.ends_with("then")
                && node_kind == "ERROR"
            {
                state.request_newline();
                state.conditional_base_indent = Some(state.indent_level);
                state.indent_level += 1;
                state.in_conditional_context = false; // Reset after handling
            }

            // Special handling for if_statement blocks
            if node_kind == "if_statement" && child_kind == "then_block" {
                // Ensure space before then_block
                state.request_space();
            }

            // Special handling for arrow (=>) - check if this should be a single-line function
            if child_kind == "=>" {
                handle_token_spacing(child_node, state, first_child);
                if let Ok(text) = child_node.utf8_text(source) {
                    state.write_token(text, opts);

                    // Check if this is a simple single-line function body by looking ahead
                    let should_be_single_line = {
                        let mut temp_cursor = cursor.node().walk();
                        temp_cursor.goto_first_child();
                        while temp_cursor.node().kind() != "=>" {
                            if !temp_cursor.goto_next_sibling() {
                                break;
                            }
                        }
                        if temp_cursor.goto_next_sibling() {
                            let next_node = temp_cursor.node();
                            next_node.child_count() <= 2
                                && next_node
                                    .utf8_text(source)
                                    .is_ok_and(|text| text.len() < 40 && !text.contains('\n'))
                        } else {
                            false
                        }
                    };

                    if should_be_single_line {
                        state.request_space();
                    } else {
                        state.request_newline();
                        state.indent_level += 1; // Increase indent for method body
                    }
                    found_arrow = true;
                }
                state.last_byte = child_node.end_byte();
            } else if child_kind == "then_block" || child_kind == "else_block" {
                // Handle then/else blocks with special newline and indentation
                if cursor.goto_first_child() {
                    let mut is_first_in_block = true;
                    loop {
                        let grandchild = cursor.node();
                        let grandchild_kind = grandchild.kind();

                        if grandchild_kind == "then" || grandchild_kind == "else" {
                            // Write the keyword at current level
                            format_node_impl(cursor, source, state, opts, is_first_in_block);
                        } else if grandchild_kind == "block" {
                            // Content block should be indented and on new line
                            state.request_newline();
                            let original_indent = state.indent_level;
                            state.indent_level += 1;
                            format_node_impl(cursor, source, state, opts, false);
                            state.indent_level = original_indent;
                        } else {
                            // Other content
                            format_node_impl(cursor, source, state, opts, is_first_in_block);
                        }

                        is_first_in_block = false;
                        if !cursor.goto_next_sibling() {
                            break;
                        }
                    }
                    cursor.goto_parent();
                } else {
                    // Fallback
                    format_node_impl(cursor, source, state, opts, first_child);
                }
            } else {
                format_node_impl(cursor, source, state, opts, first_child);
            }

            first_child = false;
            if !cursor.goto_next_sibling() {
                break;
            }
        }

        // Restore indent level after processing method body or class body
        if found_arrow {
            state.indent_level = state.indent_level.saturating_sub(1);
        }

        // Reset indent level after class/actor/trait definition
        if in_class_body {
            state.indent_level = 0;
        }

        cursor.goto_parent();
    }
}

fn handle_token_spacing(node: Node, state: &mut FormatterState, is_first: bool) {
    let kind = node.kind();

    match kind {
        // No space before these punctuation tokens
        "(" | ")" | "[" | "]" | ";" | "." | ":" => {}

        // Comma needs space after (we'll handle this by adding space for next token)
        "," => {}

        // Assignment operator needs space before and after
        "=" => {
            if !is_first {
                state.request_space();
            }
        }

        // String quotes - no space before or after
        "\"" => {
            // No additional spacing for quotes
        }

        // Identifiers and literals need context-aware spacing
        "identifier" | "number" => {
            if !is_first && !state.needs_newline {
                if let Some(last_char) = state.output.chars().last() {
                    match last_char {
                        // Add space after these
                        ',' | '=' | '+' | '-' | '*' | '/' => {
                            state.request_space();
                        }
                        // Add space if last was alphanumeric (keyword/identifier)
                        c if c.is_alphanumeric() => {
                            state.request_space();
                        }
                        // No space after these
                        '.' | '"' | '(' => {}
                        _ => {}
                    }
                }
            }
        }

        // String content should not have extra spacing
        "string_content" => {}

        // String literals need careful spacing
        "string_literal" => {
            if !is_first {
                if let Some(last_char) = state.output.chars().last() {
                    match last_char {
                        // Add space after these
                        ',' | '=' | '+' | '-' | '*' | '/' => {
                            state.request_space();
                        }
                        // Add space if last was alphanumeric (keyword/identifier)
                        c if c.is_alphanumeric() => {
                            state.request_space();
                        }
                        // No space after these
                        '.' | '"' | '(' => {}
                        _ => {}
                    }
                }
            }
        }

        // Operators usually need space
        "+" | "-" | "*" | "/" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "|" => {
            if !is_first {
                state.request_space();
            }
        }

        // Arrow operator needs space before
        "=>" => {
            if !is_first {
                state.request_space();
            }
        }

        // Keywords and boolean literals need space before them
        "actor" | "class" | "trait" | "new" | "fun" | "be" | "if" | "else" | "while" | "for"
        | "match" | "let" | "var" | "true" | "false" | "and" | "or" | "not" | "primitive"
        | "use" | "is" | "val" => {
            if !is_first {
                state.request_space();
            }
        }

        // Special handling for "then" - add space before and set conditional context for ERROR nodes
        "then" => {
            if !is_first {
                state.request_space();
            }
            // Set conditional context flag for indentation in ERROR nodes
            state.in_conditional_context = true;
        }

        // Special handling for "end" - restore indentation for ERROR nodes
        "end" => {
            if !is_first {
                state.request_newline();
            }
            // For ERROR nodes, restore the saved base indentation level
            if let Some(base_indent) = state.conditional_base_indent {
                state.indent_level = base_indent;
                state.conditional_base_indent = None;
            }
        }

        _ => {
            // Default case: check if we should add space
            if !is_first && should_add_space_before_default(state) {
                state.request_space();
            }
        }
    }
}

fn should_add_space_before_identifier(state: &FormatterState) -> bool {
    if let Some(last_char) = state.output.chars().last() {
        match last_char {
            // Add space after these characters
            '(' | '[' | ',' | ':' | '=' | '+' | '-' | '*' | '/' | '<' | '>' => true,
            // Don't add space after these
            '.' | '"' => false,
            // Add space if the last character is alphanumeric (identifiers, keywords)
            c if c.is_alphanumeric() => true,
            _ => false,
        }
    } else {
        false
    }
}

fn should_add_space_before_default(state: &FormatterState) -> bool {
    // Conservative spacing for unknown tokens
    if let Some(last_char) = state.output.chars().last() {
        matches!(last_char, ' ' | '\t') // Only if we already have whitespace
    } else {
        false
    }
}

fn handle_child_spacing(node: Node, state: &mut FormatterState) {
    let kind = node.kind();

    match kind {
        // No space before punctuation
        "(" | ")" | "[" | "]" | "," | ";" | "." | ":" => {}

        // Arrow operator gets space before
        "=>" => {
            state.request_space();
        }

        // Type annotations: space after colon
        _ if matches!(kind, "base_type" | "type") => {
            // Check if previous token was a colon
            if state.output.ends_with(':') {
                state.request_space();
            }
        }

        // Block nodes (like condition blocks) need space after keywords
        "block" => {
            // Add space if the last token was a keyword like "if"
            if let Some(last_chars) = state.output.get(state.output.len().saturating_sub(2)..) {
                if last_chars == "if" {
                    state.request_space();
                }
            }
        }

        // Most identifiers and keywords benefit from spacing in certain contexts
        "identifier" | "new" | "fun" | "be" | "if" | "then" | "else" | "while" | "for"
        | "match" | "end" | "let" | "var" | "true" | "false" | "and" | "or" | "not" => {
            if !state.needs_newline && should_add_space_before_identifier(state) {
                state.request_space();
            }
        }

        _ => {
            // Default: minimal spacing
        }
    }
}

fn handle_single_line_arguments(
    cursor: &mut TreeCursor,
    source: &[u8],
    state: &mut FormatterState,
    opts: &FormatOptions,
) {
    // Force all arguments to be on a single line with proper spacing
    if cursor.goto_first_child() {
        let mut is_first = true;
        let mut tokens = Vec::new();

        // Collect all tokens first
        loop {
            let child_node = cursor.node();
            if let Ok(text) = child_node.utf8_text(source) {
                tokens.push((child_node.kind(), text.to_owned()));
            }

            if !cursor.goto_next_sibling() {
                break;
            }
        }

        // Format tokens with proper spacing, ignoring original newlines
        for (kind, text) in tokens {
            if !is_first {
                match kind {
                    "(" | ")" | "\"" => {} // No space before punctuation and quotes
                    "," => {}              // No space before comma
                    _ => {
                        // Only add space if the previous token was not an opening paren
                        if !state.output.ends_with('(') {
                            state.request_space();
                        }
                    }
                }
            }

            state.write_token(&text, opts);

            // Add space after comma
            if kind == "," {
                state.request_space();
            }

            is_first = false;
        }

        cursor.goto_parent();
    }
}
