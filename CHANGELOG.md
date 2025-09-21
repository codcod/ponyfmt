# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- Initial release of PonyFmt experimental formatter
- Tree-sitter based Pony language parsing
- CLI with `fmt` and `debug` subcommands
- Multiple output modes: stdout, write, check
- Parallel processing with Rayon
- Configurable indentation width
- Basic indentation and spacing rules
- Comprehensive documentation and examples

### Known Issues

- Formatting is not idempotent
- Limited comment preservation
- Basic error recovery for malformed source

## [0.1.0] - TBD

### Added

- Initial public release
- Core formatting functionality
- CLI interface
- Library API
- Documentation and examples
