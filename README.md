# PonyFmt - Experimental Pony Code Formatter

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

An experimental code formatter for the [Pony programming
language](https://www.ponylang.io/) written in Rust. PonyFmt uses
[tree-sitter](https://tree-sitter.github.io/tree-sitter/) with the
[tree-sitter-pony](https://github.com/tree-sitter-grammars/tree-sitter-pony)
grammar to parse source code into an AST and apply consistent formatting rules.

## Status

**Early prototype** - This formatter implements basic indentation and whitespace
*normalization but is not yet idempotent or specification-complete. Use with
*caution on production code and always backup your files.

### Current Limitations

- Formatting is **not idempotent** (running multiple times may produce different
results)

- Comment positioning may not be preserved perfectly

- Limited error recovery for malformed source code

- Not all Pony language constructs are optimally formatted

## Features

- **Tree-sitter parsing** - Reliable syntax-aware formatting

- **Parallel processing** - Fast formatting of multiple files with Rayon

- **Multiple modes** - Print to stdout, write files in-place, or check
formatting

- ️**Configurable indentation** - Customize indentation width

- **Directory support** - Format entire directories recursively

## Installation

### From Source

```bash
git clone https://github.com/yourusername/ponyfmt.git

cd ponyfmt

cargo install --path .
```

## Usage

### Command Line Interface

PonyFmt provides a command-line interface with several modes of operation:

#### Basic Formatting

Format all `.pony` files in the current directory and print to stdout:

```bash
ponyfmt fmt
```

Format specific files or directories:

```bash
ponyfmt fmt src/main.pony
ponyfmt fmt src/ examples/
```

#### Check Mode (CI-Friendly)

Check if files are properly formatted without making changes. Returns non-zero
exit code if formatting changes would be made:

```bash
ponyfmt fmt --check .
ponyfmt fmt --check src/
```

This is useful for CI/CD pipelines to ensure code is properly formatted.

#### Write Mode

Write formatted changes back to files in-place:

```bash
ponyfmt fmt --write src/
ponyfmt fmt --write --indent 4 src/
```

**⚠️ Warning:** Always backup your files before using `--write` mode, as
*formatting is not yet idempotent.

#### Custom Indentation

Specify the number of spaces for indentation:

```bash
ponyfmt fmt --indent 2 src/        # 2 spaces (default)
ponyfmt fmt --indent 4 --write src/ # 4 spaces, write changes
```

#### Debug Mode

Inspect the AST structure of Pony files:

```bash
ponyfmt debug src/main.pony
```

This prints the tree-sitter AST for debugging formatter behavior.

### Library Usage

PonyFmt can also be used as a Rust library in your own projects.

Add to your `Cargo.toml`:

```toml
[dependencies]
ponyfmt = "0.1"
```

#### Basic Library Usage

```rust
use ponyfmt::formatter::{FormatOptions, Mode, format_source};

let unformatted_code = r#"
actor Main
new create(env:Env)=>
env.out.print("Hello, World!")
"#;

let options = FormatOptions {
    indent_width: 2,
    mode: Mode::Stdout,
};

match format_source(unformatted_code, &options) {
    Ok(formatted) => println!("{}", formatted),
    Err(e) => eprintln!("Formatting error: {}", e),
}
```

#### Advanced Library Usage

```rust
use ponyfmt::formatter::{FormatOptions, Mode, format_source};
use std::fs;

// Read a Pony file
let source = fs::read_to_string("src/main.pony")?;

// Configure formatting options
let options = FormatOptions {
    indent_width: 4,
    mode: Mode::Write,  // Although mode doesn't affect format_source output
};

// Format the code
let formatted = format_source(&source, &options)?;

// Write back to file (if desired)
fs::write("src/main.pony", formatted)?;
```

### Integration Examples

#### CI/CD Pipeline (GitHub Actions)

```yaml
name: Check Formatting
on: [push, pull_request]

jobs:
  format-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Install ponyfmt
        run: cargo install ponyfmt
      - name: Check formatting
        run: ponyfmt fmt --check .
```

#### Pre-commit Hook

```bash
#!/bin/sh
# .git/hooks/pre-commit
ponyfmt fmt --check . || {
    echo "Code is not formatted. Run 'ponyfmt fmt --write .' to fix."
    exit 1
}
```

#### Makefile Integration

```makefile
.PHONY: format format-check

format:
    ponyfmt fmt --write src/

format-check:
    ponyfmt fmt --check src/

test: format-check
    # run your tests here
```

### Command Reference

```text
ponyfmt fmt [OPTIONS] [PATHS]...

ARGUMENTS:
    [PATHS]...    Files or directories to format (default: current directory)

OPTIONS:
    --write       Write formatted output back to files
    --check       Check if files are formatted (exit 1 if not)
    --indent <N>  Number of spaces for indentation (default: 2)
    -h, --help    Print help information

ponyfmt debug <FILE>

ARGUMENTS:
    <FILE>        Pony file to debug

OPTIONS:
    -h, --help    Print help information
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file
for details.

## Acknowledgments

- [Pony Language](https://www.ponylang.io/)
- [tree-sitter](https://tree-sitter.github.io/tree-sitter/)
- [tree-sitter-pony](https://github.com/tree-sitter-grammars/tree-sitter-pony)
