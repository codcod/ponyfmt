use ponyfmt::formatter::{format_source, FormatOptions, Mode};
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

fn fmt(src: &str) -> String {
    let opts = FormatOptions {
        mode: Mode::Stdout, // irrelevant for format_source
        ..FormatOptions::default()
    };
    format_source(src, &opts).unwrap()
}

/// Find all test cases in the examples directory
fn find_test_cases() -> Vec<TestCase> {
    let examples_dir = Path::new("tests/examples");
    let mut test_cases = Vec::new();

    // Walk through all directories including subdirectories
    for entry in WalkDir::new(examples_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();

        // Skip directories
        if !path.is_file() {
            continue;
        }

        // Look for input files with new pattern: *_*.input
        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
            if file_name.ends_with(".input") && file_name.contains('_') {
                if let Some(test_case) = create_test_case(path) {
                    test_cases.push(test_case);
                }
            }
        }
    }

    test_cases
}

#[derive(Debug, Clone)]
struct TestCase {
    name: String,
    input_file: PathBuf,
    expected_file: PathBuf,
}

/// Create a test case from an input file path
fn create_test_case(input_path: &Path) -> Option<TestCase> {
    let file_name = input_path.file_name()?.to_str()?;
    let parent_dir = input_path.parent()?;

    // Extract the base name from the input file
    // e.g., "simple_1.input" -> "simple"
    // e.g., "example_2.input" -> "example"
    let base_name = if let Some(pos) = file_name.rfind('_') {
        // Get everything before the last underscore
        &file_name[..pos]
    } else {
        return None;
    };

    // Look for the corresponding expected file: base_name.pony
    let expected_file = parent_dir.join(format!("{}.pony", base_name));

    if expected_file.exists() {
        Some(TestCase {
            name: format!("{}_{}", base_name, input_path.display()),
            input_file: input_path.to_path_buf(),
            expected_file,
        })
    } else {
        None
    }
}

/// Run a single test case
fn run_test_case(test_case: &TestCase) -> Result<(), String> {
    // Read input file
    let input_content = fs::read_to_string(&test_case.input_file).map_err(|e| {
        format!(
            "Failed to read input file {:?}: {}",
            test_case.input_file, e
        )
    })?;

    // Read expected file
    let expected_content = fs::read_to_string(&test_case.expected_file).map_err(|e| {
        format!(
            "Failed to read expected file {:?}: {}",
            test_case.expected_file, e
        )
    })?;

    // Format the input
    let formatted_content = fmt(&input_content);

    // Compare results
    if formatted_content.trim() == expected_content.trim() {
        Ok(())
    } else {
        Err(format!(
            "Formatting mismatch for test case '{}':\n\
             Input file: {:?}\n\
             Expected file: {:?}\n\
             \n--- Expected ---\n{}\n\
             \n--- Got ---\n{}\n\
             \n--- Diff ---\n{}",
            test_case.name,
            test_case.input_file,
            test_case.expected_file,
            expected_content,
            formatted_content,
            create_diff(&expected_content, &formatted_content)
        ))
    }
}

/// Create a simple diff visualization
fn create_diff(expected: &str, actual: &str) -> String {
    let expected_lines: Vec<&str> = expected.lines().collect();
    let actual_lines: Vec<&str> = actual.lines().collect();

    let mut diff = String::new();
    let max_lines = expected_lines.len().max(actual_lines.len());

    for i in 0..max_lines {
        let expected_line = expected_lines.get(i).unwrap_or(&"");
        let actual_line = actual_lines.get(i).unwrap_or(&"");

        if expected_line != actual_line {
            diff.push_str(&format!(
                "Line {}: Expected: {:?}, Got: {:?}\n",
                i + 1,
                expected_line,
                actual_line
            ));
        }
    }

    if diff.is_empty() {
        "No line differences (possibly trailing whitespace)".to_string()
    } else {
        diff
    }
}

#[test]
fn example_files_formatting() {
    let test_cases = find_test_cases();

    // Ensure we found at least some test cases
    assert!(
        !test_cases.is_empty(),
        "No test cases found in tests/examples/"
    );

    println!("Found {} test case(s):", test_cases.len());
    for test_case in &test_cases {
        println!("  - {}", test_case.name);
    }

    // Run all test cases
    let mut failures = Vec::new();
    for test_case in &test_cases {
        if let Err(error) = run_test_case(test_case) {
            failures.push(error);
        }
    }

    // Report failures
    if !failures.is_empty() {
        panic!("Test failures:\n\n{}", failures.join("\n\n"));
    }
}

#[test]
fn basic_actor_formatting() {
    let input = r#"actor Main
new create(env: Env) =>
env.out.print("Hi")
"#;
    let expected_indent2 = r#"actor Main
  new create(env: Env) =>
    env.out.print("Hi")
"#;
    assert_eq!(fmt(input), expected_indent2);
}

#[test]
fn conditional_formatting() {
    let input = r#"if true then
env.out.print("yes")
end"#;
    let expected = r#"if true then
  env.out.print("yes")
end
"#;
    assert_eq!(fmt(input), expected);
}

#[test]
fn debug_example_case() {
    let input = r#"actor Main
new create(env: Env) =>
env.out.print("Hello World")
fun demo() =>
if true then
env.out.print("nested")
end"#;

    let result = fmt(input);
    println!("Result:\n{}", result);
    // Just test that it runs without panicking for now
}
