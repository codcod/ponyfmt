#[cfg(test)]
mod performance_debug {
    use crate::formatter::{FormatOptions, Mode, format_source};
    use std::fs;
    use std::time::Instant;

    #[test]
    fn debug_single_file_performance() {
        let file_path = "tests/examples/pony-express/src/message_1.input";

        println!("Testing file: {}", file_path);
        let start = Instant::now();

        let content = fs::read_to_string(file_path).expect("Failed to read file");

        let opts = FormatOptions {
            mode: Mode::Stdout,
            ..FormatOptions::default()
        };

        let result = format_source(&content, &opts);
        let duration = start.elapsed();

        println!("Formatting took: {:?}", duration);

        match result {
            Ok(_formatted) => {
                println!("✓ Formatted successfully");
                // Don't print the full output to avoid clutter
            }
            Err(e) => {
                println!("✗ Error: {}", e);
                panic!("Formatting failed");
            }
        }

        // Fail if it takes more than 5 seconds
        assert!(
            duration.as_secs() < 5,
            "Formatting took too long: {:?}",
            duration
        );
    }
}
