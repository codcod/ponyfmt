//! Core formatting engine for Pony source code
//!
//! This module contains the main formatting logic that transforms parsed Pony AST
//! nodes into formatted source code following Pony conventions:
//! - 2-space indentation for all nested content
//! - Blank lines after block comments
//! - Proper spacing around operators and keywords
//! - Class/actor members indented within their containers

use crate::parser::parse;
use anyhow::Result;
use tree_sitter::Node;

/// Output mode for the formatter
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
pub struct FormatOptions {
    /// Number of spaces to use for each indentation level (defaults to 2 for Pony)
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

/// Simple formatter state that tracks indentation and output
#[derive(Debug)]
struct FormatterState {
    output: String,
    indent_level: usize,
    current_line_has_content: bool,
}

impl FormatterState {
    fn new() -> Self {
        Self {
            output: String::new(),
            indent_level: 0,
            current_line_has_content: false,
        }
    }

    fn write_indent(&mut self, opts: &FormatOptions) {
        if !self.current_line_has_content {
            for _ in 0..(self.indent_level * opts.indent_width) {
                self.output.push(' ');
            }
        }
    }

    fn write_text(&mut self, text: &str) {
        self.output.push_str(text);
        self.current_line_has_content = true;
    }

    fn write_newline(&mut self) {
        self.output.push('\n');
        self.current_line_has_content = false;
    }

    fn write_blank_line(&mut self) {
        if self.current_line_has_content {
            self.write_newline();
        }
        self.write_newline();
    }

    fn increase_indent(&mut self) {
        self.indent_level += 1;
    }

    fn decrease_indent(&mut self) {
        if self.indent_level > 0 {
            self.indent_level -= 1;
        }
    }
}

/// Format Pony source code according to style conventions
pub fn format_source(input: &str, opts: &FormatOptions) -> Result<String> {
    let tree = parse(input)?;
    let root_node = tree.root_node();
    let mut state = FormatterState::new();

    format_node(root_node, input.as_bytes(), &mut state, opts);

    Ok(state.output)
}

fn format_arguments(node: Node, source: &[u8], state: &mut FormatterState, _opts: &FormatOptions) {
    // Extract all the text content and reformat on a single line
    let full_text = node_text(node, source);

    // Remove the parentheses and extract the content
    let content = full_text.strip_prefix('(').unwrap_or(&full_text);
    let content = content.strip_suffix(')').unwrap_or(content);

    // Split by commas and clean up each argument
    let args: Vec<&str> = content
        .split(',')
        .map(|arg| arg.trim())
        .filter(|arg| !arg.is_empty())
        .collect();

    // Write formatted arguments
    state.write_text("(");
    for (i, arg) in args.iter().enumerate() {
        if i > 0 {
            state.write_text(", ");
        }
        state.write_text(arg);
    }
    state.write_text(")");
}

fn format_parameters(node: Node, source: &[u8], state: &mut FormatterState, _opts: &FormatOptions) {
    // Similar to format_arguments but handles parameter declarations
    let full_text = node_text(node, source);

    // Remove the parentheses and extract the content
    let content = full_text.strip_prefix('(').unwrap_or(&full_text);
    let content = content.strip_suffix(')').unwrap_or(content);

    // Split by commas and clean up each parameter
    let params: Vec<&str> = content
        .split(',')
        .map(|param| param.trim())
        .filter(|param| !param.is_empty())
        .collect();

    // Write formatted parameters
    state.write_text("(");
    for (i, param) in params.iter().enumerate() {
        if i > 0 {
            state.write_text(", ");
        }
        state.write_text(param);
    }
    state.write_text(")");
}

fn format_node(node: Node, source: &[u8], state: &mut FormatterState, opts: &FormatOptions) {
    match node.kind() {
        "source_file" => {
            // Handle the root of the file

            let mut prev_kind: Option<&str> = None;

            for child in node.children(&mut node.walk()) {
                let current_kind = child.kind();

                // Add blank lines between different types of top-level declarations
                if let Some(prev) = prev_kind {
                    let needs_blank_line = match (prev, current_kind) {
                        // Always add blank line after block comments (unless next is also a comment)
                        ("block_comment", kind)
                            if kind != "block_comment" && kind != "line_comment" =>
                        {
                            true
                        }
                        // Add blank lines between different declaration types
                        ("primitive_definition", kind)
                            if kind != "primitive_definition" && kind != "line_comment" =>
                        {
                            true
                        }
                        ("type_alias", kind) if kind != "type_alias" && kind != "line_comment" => {
                            true
                        }
                        ("trait_definition", kind)
                            if kind != "trait_definition" && kind != "line_comment" =>
                        {
                            true
                        }
                        ("class_definition", kind) if kind != "line_comment" => true,
                        // Add blank line before line comments that come after class definitions
                        ("class_definition", "line_comment")
                        | ("trait_definition", "line_comment")
                        | ("primitive_definition", "line_comment") => true,
                        // Add blank line before first declaration after use statements
                        ("use_statement", kind)
                            if kind != "use_statement" && kind != "line_comment" =>
                        {
                            true
                        }
                        _ => false,
                    };

                    if needs_blank_line {
                        state.write_blank_line();
                    }
                }

                format_node(child, source, state, opts);
                prev_kind = Some(current_kind);
            }
        }

        "block_comment" | "line_comment" => {
            let text = node_text(node, source);
            state.write_indent(opts);
            state.write_text(&text);
            state.write_newline();
        }

        "use_statement" => {
            state.write_indent(opts);
            state.write_text("use ");

            // Handle different types of use statements
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                // Skip the "use" keyword we already wrote
                if node_text(cursor.node(), source) == "use" {
                    cursor.goto_next_sibling();
                }

                // Format the rest of the use statement
                while cursor.node().kind() != "" {
                    let child = cursor.node();
                    match child.kind() {
                        "string" => {
                            state.write_text(&node_text(child, source));
                        }
                        _ => {
                            // For complex use statements like @printf[I32](...), format as-is
                            state.write_text(&node_text(child, source));
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
            state.write_newline();
        }

        "actor_definition" | "class_definition" => {
            state.write_indent(opts);

            // Handle actor/class keyword and name
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "actor" | "class" => {
                            state.write_text(&node_text(child, source));
                            state.write_text(" ");
                        }
                        "capability" => {
                            state.write_text(&node_text(child, source));
                            state.write_text(" ");
                        }
                        "identifier" => {
                            // This is the type name
                            state.write_text(&node_text(child, source));
                        }
                        "is" => {
                            state.write_text(" is ");
                        }
                        "base_type" => {
                            // This is the parent type name
                            state.write_text(&node_text(child, source));
                        }
                        "members" => {
                            // Now handle the body
                            state.write_newline();
                            state.increase_indent();
                            format_node(child, source, state, opts);
                            state.decrease_indent();
                        }
                        _ => {}
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        "trait_definition" => {
            state.write_indent(opts);

            // Handle trait keyword and name
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "trait" => {
                            state.write_text(&node_text(child, source));
                            state.write_text(" ");
                        }
                        "capability" => {
                            state.write_text(&node_text(child, source));
                            state.write_text(" ");
                        }
                        "identifier" => {
                            state.write_text(&node_text(child, source));
                        }
                        "members" => {
                            state.write_newline();
                            state.increase_indent();
                            format_node(child, source, state, opts);
                            state.decrease_indent();
                        }
                        _ => {}
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        "primitive_definition" => {
            state.write_indent(opts);

            let mut has_members = false;
            // Handle primitive keyword and name
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "primitive" => {
                            state.write_text(&node_text(child, source));
                            state.write_text(" ");
                        }
                        "identifier" => {
                            state.write_text(&node_text(child, source));
                        }
                        "members" => {
                            // Handle primitive members (methods, etc.)
                            state.write_newline();
                            state.increase_indent();
                            format_node(child, source, state, opts);
                            state.decrease_indent();
                            has_members = true;
                        }
                        _ => {}
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }

            // Always add newline after primitive (whether it has members or not)
            if !has_members {
                state.write_newline();
            }
        }

        "type_definition" => {
            state.write_indent(opts);
            state.write_text("type ");

            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    if child.kind() == "identifier" {
                        state.write_text(&node_text(child, source));
                        break;
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }

            // Find the "is" part and the type union
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    if child.kind() == "is" {
                        state.write_text(" is ");
                        // Find the next significant node (the type union)
                        if cursor.goto_next_sibling() {
                            let union_node = cursor.node();
                            state.write_text(&node_text(union_node, source));
                        }
                        break;
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
            state.write_newline();
        }

        "type_alias" => {
            state.write_indent(opts);
            state.write_text("type ");

            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "identifier" => {
                            state.write_text(&node_text(child, source));
                        }
                        "is" => {
                            state.write_text(" is ");
                        }
                        "union_type" => {
                            state.write_text(&node_text(child, source));
                        }
                        _ => {}
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
            state.write_newline();
        }
        "members" => {
            // Handle members of a type (fields, constructors, functions)
            let mut prev_significant_kind: Option<&str> = None;

            for child in node.children(&mut node.walk()) {
                let current_kind = child.kind();

                // Skip comments when determining spacing
                if current_kind == "line_comment" || current_kind == "block_comment" {
                    format_node(child, source, state, opts);
                    continue;
                }

                if let Some(prev) = prev_significant_kind {
                    let needs_blank_line = match (prev, current_kind) {
                        // Blank line after the last field, before the first method/constructor
                        ("field" | "field_definition", "constructor" | "method" | "behavior") => {
                            true
                        }
                        // Blank line between any two methods/constructors/behaviors
                        (
                            "constructor" | "method" | "behavior",
                            "constructor" | "method" | "behavior",
                        ) => true,
                        // No blank line between consecutive fields or other cases
                        _ => false,
                    };

                    if needs_blank_line {
                        state.write_blank_line();
                    }
                }

                format_node(child, source, state, opts);
                prev_significant_kind = Some(current_kind);
            }
        }

        "field" => {
            state.write_indent(opts);

            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "let" | "var" | "embed" => {
                            state.write_text(&node_text(child, source));
                            state.write_text(" ");
                        }
                        "identifier" => {
                            state.write_text(&node_text(child, source));
                        }
                        ":" => {
                            state.write_text(": ");
                        }
                        "base_type" => {
                            state.write_text(&node_text(child, source));
                        }
                        "=" => {
                            state.write_text(" = ");
                        }
                        _ => {
                            // For default values
                            if child.kind() != "let"
                                && child.kind() != "var"
                                && child.kind() != "embed"
                                && child.kind() != "identifier"
                                && child.kind() != ":"
                                && child.kind() != "="
                                && child.kind() != "base_type"
                            {
                                state.write_text(&node_text(child, source));
                            }
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
            state.write_newline();
        }

        "field_definition" => {
            state.write_indent(opts);

            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "let" | "var" | "embed" => {
                            state.write_text(&node_text(child, source));
                            state.write_text(" ");
                        }
                        "identifier" => {
                            state.write_text(&node_text(child, source));
                        }
                        ":" => {
                            state.write_text(": ");
                        }
                        "=" => {
                            state.write_text(" = ");
                        }
                        _ => {
                            // For type annotations and default values
                            if child.kind() != "let"
                                && child.kind() != "var"
                                && child.kind() != "embed"
                                && child.kind() != "identifier"
                                && child.kind() != ":"
                                && child.kind() != "="
                            {
                                state.write_text(&node_text(child, source));
                            }
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
            state.write_newline();
        }
        "method" | "constructor" | "function_definition" | "behavior" => {
            state.write_indent(opts);

            let mut cursor = node.walk();
            let mut found_colon = false;
            let mut is_simple_block = false;

            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "new" | "fun" | "be" => {
                            state.write_text(&node_text(child, source));
                            state.write_text(" ");
                        }
                        "capability" => {
                            state.write_text(&node_text(child, source));
                            state.write_text(" ");
                        }
                        "identifier" => {
                            state.write_text(&node_text(child, source));
                        }
                        "parameters" => {
                            format_parameters(child, source, state, opts);
                        }
                        ":" => {
                            state.write_text(": ");
                            found_colon = true;
                        }
                        "=>" => {
                            state.write_text(" =>");
                        }
                        "block" => {
                            let block_text_owned = node_text(child, source);
                            let block_text = block_text_owned.trim();
                            is_simple_block = !block_text.contains('\n') && block_text.len() < 50;

                            if is_simple_block {
                                state.write_text(" ");
                                state.write_text(block_text);
                            } else {
                                state.write_newline();
                                state.increase_indent();
                                format_node(child, source, state, opts);
                                state.decrease_indent();
                            }
                        }
                        _ => {
                            if found_colon && child.kind() != "=>" && child.kind() != "block" {
                                format_node(child, source, state, opts);
                            }
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
            if !is_simple_block {
                state.write_newline();
            }
        }

        "if_statement" => {
            state.write_indent(opts);

            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "if_block" => {
                            // Handle "if condition"
                            format_if_block(child, source, state, opts);
                        }
                        "then_block" => {
                            // Handle "then body"
                            format_then_block(child, source, state, opts);
                        }
                        "end" => {
                            state.write_indent(opts);
                            state.write_text("end");
                            state.write_newline();
                        }
                        _ => {}
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        "block" => {
            // For method bodies and statement blocks, format each child with proper indentation
            for child in node.children(&mut node.walk()) {
                // Skip empty nodes that can appear
                if child.is_named() {
                    state.write_indent(opts);
                    format_node(child, source, state, opts);
                    state.write_newline();
                }
            }
        }

        "call_expression" => {
            // Format function calls more intelligently
            for child in node.children(&mut node.walk()) {
                match child.kind() {
                    "member_expression" => {
                        // This is the function name (e.g., EmailMessage.create)
                        state.write_text(&node_text(child, source));
                    }
                    "ffi_identifier" => {
                        // This is an FFI call like @printf
                        state.write_text(&node_text(child, source));
                    }
                    "identifier" => {
                        // This is a simple function call (no member access)
                        state.write_text(&node_text(child, source));
                    }
                    "arguments" => {
                        // Format arguments, keeping simple calls on one line
                        format_arguments(child, source, state, opts);
                    }
                    _ => {
                        // Handle other components as needed
                        format_node(child, source, state, opts);
                    }
                }
            }
        }

        "assignment_expression" => {
            // Single assignment, format properly
            state.write_indent(opts);

            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "variable_declaration" => {
                            // Handle let/var declarations properly
                            format_node(child, source, state, opts);
                        }
                        "identifier" => {
                            state.write_text(&node_text(child, source));
                        }
                        "=" => {
                            state.write_text(" = ");
                        }
                        _ => {
                            // Format the right-hand side expression
                            format_node(child, source, state, opts);
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
            state.write_newline();
        }

        "variable_declaration" => {
            // Handle let/var declarations
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "let" | "var" => {
                            state.write_text(&node_text(child, source));
                            state.write_text(" ");
                        }
                        "identifier" => {
                            state.write_text(&node_text(child, source));
                        }
                        ":" => {
                            state.write_text(": ");
                        }
                        _ => {
                            // Type annotations, etc.
                            format_node(child, source, state, opts);
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        "assignment" => {
            state.write_indent(opts);
            state.write_text(&node_text(node, source));
            state.write_newline();
        }

        "identifier" => {
            // Handle identifiers - this is needed for standalone identifiers
            state.write_text(&node_text(node, source));
        }

        // Skip these nodes as they are handled by their parents
        "actor" | "class" | "trait" | "primitive" | "new" | "fun" | "be" | "let" | "var"
        | "embed" | "parameters" | ":" | "=>" | "val" | "ref" | "iso" | "trn" | "box" | "tag"
        | "(" | ")" | "=" | "if" | "then" | "end" | "is" | "capability" | "match" | "try"
        | "else" | "return" | "as" | "." | "string_content" | "\"" => {
            // These are handled by their parent nodes
        }

        "ERROR" => {
            // Handle ERROR nodes by processing their children with basic formatting
            let text = node_text(node, source);

            // Check if this looks like the start of an if statement
            if text.starts_with("if ") && text.contains("then") {
                // Format as if statement start
                state.write_indent(opts);
                let parts: Vec<&str> = text.splitn(2, "then").collect();
                if parts.len() == 2 {
                    state.write_text(&format!("{} then", parts[0].trim()));
                    state.write_newline();
                    state.increase_indent();
                    // Format the rest as body content
                    if !parts[1].trim().is_empty() {
                        state.write_indent(opts);
                        state.write_text(parts[1].trim());
                    }
                }
            } else if text.trim() == "end" || text.ends_with("end") {
                // Format as if statement end
                if text.trim() != "end" {
                    // There's content before "end" (like ")end")
                    let content = text.trim_end_matches("end").trim();
                    if !content.is_empty() {
                        state.write_text(content);
                        state.write_newline();
                    }
                }
                state.decrease_indent();
                state.write_indent(opts);
                state.write_text("end");
                state.write_newline();
            } else {
                // For other ERROR nodes, just format the content with indentation
                state.write_indent(opts);
                state.write_text(&text);
                state.write_newline();
            }
        }

        "string" => {
            // Handle string literals
            state.write_text(&node_text(node, source));
        }

        "member_expression" => {
            // Handle member access like Logger.print or msg.original
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "." => {
                            state.write_text(".");
                        }
                        _ => {
                            format_node(child, source, state, opts);
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        "match_expression" => {
            state.write_indent(opts);

            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "match" => {
                            state.write_text("match ");
                        }
                        "end" => {
                            state.write_indent(opts);
                            state.write_text("end");
                            state.write_newline();
                        }
                        _ => {
                            // Handle match expression, cases, etc.
                            format_node(child, source, state, opts);
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        "match_case" => {
            state.write_indent(opts);
            state.write_text("| ");

            let mut cursor = node.walk();
            let mut found_arrow = false;

            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "=>" => {
                            state.write_text(" => ");
                            found_arrow = true;
                        }
                        _ => {
                            if found_arrow {
                                // This is the case body
                                format_node(child, source, state, opts);
                            } else {
                                // This is the pattern
                                state.write_text(&node_text(child, source));
                            }
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
            state.write_newline();
        }

        "try_expression" | "try_statement" => {
            state.write_indent(opts);
            state.write_text("try");
            state.write_newline();
            state.increase_indent();

            // Format the body of the try block
            for child in node.children(&mut node.walk()) {
                if child.kind() != "try" && child.kind() != "end" {
                    format_node(child, source, state, opts);
                }
            }

            state.decrease_indent();
            state.write_indent(opts);
            state.write_text("end");
            state.write_newline();
        }

        "binary_expression" => {
            // Handle binary expressions like string concatenation
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                let mut first = true;
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "+" | "-" | "*" | "/" | "==" | "!=" | "<" | ">" | "<=" | ">=" | "and"
                        | "or" => {
                            state.write_text(" ");
                            state.write_text(&node_text(child, source));
                            state.write_text(" ");
                        }
                        _ => {
                            if !first {
                                // Operator was already written, this is the right operand
                                format_node(child, source, state, opts);
                            } else {
                                // This is the left operand
                                format_node(child, source, state, opts);
                                first = false;
                            }
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        "expression_statement" => {
            // Format expression statements with proper indentation
            state.write_indent(opts);
            for child in node.children(&mut node.walk()) {
                format_node(child, source, state, opts);
            }
            state.write_newline();
        }

        "else_clause" => {
            state.write_indent(opts);
            state.write_text("else");
            state.write_newline();
            state.increase_indent();

            for child in node.children(&mut node.walk()) {
                if child.kind() != "else" {
                    format_node(child, source, state, opts);
                }
            }

            state.decrease_indent();
        }

        "return_statement" => {
            state.write_indent(opts);
            state.write_text("return");

            // Add return value if present
            for child in node.children(&mut node.walk()) {
                if child.kind() != "return" {
                    state.write_text(" ");
                    format_node(child, source, state, opts);
                }
            }
            state.write_newline();
        }

        "as_expression" | "cast_expression" => {
            // Handle type casting expressions
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "as" => {
                            state.write_text(" as ");
                        }
                        _ => {
                            format_node(child, source, state, opts);
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        "method_declaration" => {
            // Handle trait method declarations
            state.write_indent(opts);

            let mut cursor = node.walk();
            let mut found_colon = false;

            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        "fun" | "be" => {
                            state.write_text(&node_text(child, source));
                            state.write_text(" ");
                        }
                        "capability" => {
                            state.write_text(&node_text(child, source));
                            state.write_text(" ");
                        }
                        "identifier" => {
                            state.write_text(&node_text(child, source));
                        }
                        "parameters" => {
                            format_parameters(child, source, state, opts);
                        }
                        ":" => {
                            state.write_text(": ");
                            found_colon = true;
                        }
                        _ => {
                            // Handle return type annotations
                            if found_colon {
                                format_node(child, source, state, opts);
                            }
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
            state.write_newline();
        }

        "base_type" => {
            // Handle type names
            state.write_text(&node_text(node, source));
        }

        "parameter" => {
            // Handle parameter declarations
            let mut cursor = node.walk();
            if cursor.goto_first_child() {
                loop {
                    let child = cursor.node();
                    match child.kind() {
                        ":" => {
                            state.write_text(": ");
                        }
                        _ => {
                            format_node(child, source, state, opts);
                        }
                    }
                    if !cursor.goto_next_sibling() {
                        break;
                    }
                }
            }
        }

        "arguments" => {
            // Handle function call arguments
            format_arguments(node, source, state, opts);
        }

        _ => {
            // For any unhandled nodes, recursively format children
            for child in node.children(&mut node.walk()) {
                format_node(child, source, state, opts);
            }
        }
    }
}

fn format_if_block(node: Node, source: &[u8], state: &mut FormatterState, _opts: &FormatOptions) {
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            match child.kind() {
                "if" => {
                    state.write_text("if ");
                }
                "block" => {
                    // The condition
                    state.write_text(&node_text(child, source));
                    state.write_text(" ");
                }
                _ => {}
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

fn format_then_block(node: Node, source: &[u8], state: &mut FormatterState, opts: &FormatOptions) {
    let mut cursor = node.walk();
    if cursor.goto_first_child() {
        loop {
            let child = cursor.node();
            match child.kind() {
                "then" => {
                    state.write_text("then");
                    state.write_newline();
                }
                "block" => {
                    // The then body
                    state.increase_indent();
                    format_node(child, source, state, opts);
                    state.decrease_indent();
                }
                _ => {}
            }
            if !cursor.goto_next_sibling() {
                break;
            }
        }
    }
}

fn node_text(node: Node, source: &[u8]) -> String {
    let start = node.start_byte();
    let end = node.end_byte();
    String::from_utf8_lossy(&source[start..end]).to_string()
}
