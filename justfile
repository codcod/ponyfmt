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

# Run tests with detailed output
[group('qa')]
test-basic:
    cargo test basic_actor_formatting

[group('qa')]
test-files:
    cargo test example_files_formatting -- --nocapture
