mod formatter;
mod parser;

use anyhow::{Result, bail};
use clap::{Parser, Subcommand};
use formatter::{FormatOptions, Mode, format_source};
use rayon::prelude::*;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

#[derive(Parser, Debug)]
#[command(name = "ponyfmt", version, about = "Experimental Pony formatter")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Fmt {
        /// Paths (files or directories) to format (defaults to current dir)
        paths: Vec<PathBuf>,
        /// Write the formatted content back to the files
        #[arg(long)]
        write: bool,
        /// Check if files are formatted; non-zero exit if changes needed
        #[arg(long)]
        check: bool,
        /// Indent width
        #[arg(long, default_value_t = 2)]
        indent: usize,
    },
    Debug {
        /// File to debug
        file: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match cli.command {
        Commands::Fmt {
            paths,
            write,
            check,
            indent,
        } => {
            if write && check {
                bail!("--write and --check are mutually exclusive");
            }
            let mode = if write {
                Mode::Write
            } else if check {
                Mode::Check
            } else {
                Mode::Stdout
            };
            let opts = FormatOptions {
                indent_width: indent,
                mode,
            };
            let targets = if paths.is_empty() {
                vec![PathBuf::from(".")]
            } else {
                paths
            };
            let mut pony_files = Vec::new();
            for p in targets {
                collect_pony_files(&p, &mut pony_files);
            }

            let results: Vec<_> = pony_files
                .par_iter()
                .map(|path| process_file(path, &opts))
                .collect();
            let mut had_change = false;
            for r in results {
                match r {
                    Ok(changed) => had_change |= changed,
                    Err(e) => eprintln!("{}", e),
                }
            }
            if matches!(mode, Mode::Check) && had_change {
                std::process::exit(1);
            }
        }
        Commands::Debug { file } => {
            debug_file(&file)?;
        }
    }
    Ok(())
}

fn debug_file(path: &Path) -> Result<()> {
    let content = fs::read_to_string(path)?;
    let tree = parser::parse(&content)?;
    println!("===== {} =====", path.display());
    print_tree(&tree, &content, tree.root_node(), 0);
    Ok(())
}

fn print_tree(_tree: &tree_sitter::Tree, source: &str, node: tree_sitter::Node, depth: usize) {
    let indent = "  ".repeat(depth);
    let kind = node.kind();
    let start = node.start_position();
    let end = node.end_position();

    if node.child_count() == 0 {
        let text = node.utf8_text(source.as_bytes()).unwrap_or("");
        println!(
            "{}{}@{}:{}-{}:{} '{}'",
            indent, kind, start.row, start.column, end.row, end.column, text
        );
    } else {
        println!(
            "{}{}@{}:{}-{}:{}",
            indent, kind, start.row, start.column, end.row, end.column
        );
        for i in 0..node.child_count() {
            if let Some(child) = node.child(i) {
                print_tree(_tree, source, child, depth + 1);
            }
        }
    }
}

fn collect_pony_files(path: &Path, out: &mut Vec<PathBuf>) {
    if path.is_file() {
        if path.extension().and_then(|s| s.to_str()) == Some("pony") {
            out.push(path.to_path_buf());
        }
        return;
    }
    for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
        let p = entry.path();
        if p.is_file() && p.extension().and_then(|s| s.to_str()) == Some("pony") {
            out.push(p.to_path_buf());
        }
    }
}

fn process_file(path: &Path, opts: &FormatOptions) -> Result<bool> {
    let content = fs::read_to_string(path)?;
    let formatted = format_source(&content, opts)?;
    let changed = formatted != content;
    match opts.mode {
        Mode::Stdout => {
            println!("===== {} =====", path.display());
            print!("{}", formatted);
        }
        Mode::Write => {
            if changed {
                fs::write(path, formatted)?;
            }
        }
        Mode::Check => {}
    }
    Ok(changed)
}
