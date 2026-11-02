# AGENTS.md

## Project overview

This repository is a Rust crate and CLI for image comparison and antialias detection. The public library lives in [src/lib.rs](src/lib.rs), and the CLI entry point is in [src/main.rs](src/main.rs). The main project documentation is in [README.md](README.md).

## Repository conventions

- Prefer small, focused edits that keep behavior localized to the relevant module.
- Follow the existing Rust module layout: core logic in [src/lib.rs](src/lib.rs) and feature-specific behavior in sibling modules such as [src/diff.rs](src/diff.rs), [src/validate.rs](src/validate.rs), [src/results.rs](src/results.rs), and [src/utils.rs](src/utils.rs).
- Preserve the public API shape unless the task explicitly requires a breaking change.
- For bug fixes, add or update tests close to the affected module or behavior before finalizing the change.
- Always use thiserror and add to GridSkipperError instead of using std::error.

## Build and test commands

Use the workspace root as the working directory.

- Run the full test suite: `cargo test`
- Run a targeted Rust test: `cargo test <test_name>`
- Run the CLI: `cargo run -- diff <left> <right>` or `cargo run -- detect --image-path <image> --without-aa-path <ground-truth> --output-dir <dir>`

## Architecture notes

- The crate is organized as a library plus a binary. The binary is mainly a thin CLI wrapper around library functions.
- Image processing and antialias detection are implemented in the library modules rather than in the binary entry point.
- Validation and metadata checks live in [src/validate.rs](src/validate.rs), so new image-processing code should usually validate inputs before running the algorithm.
- When working on detection logic, prefer to maintain the existing algorithm-specific split rather than reworking unrelated modules.

## Working style for AI agents

- Keep changes minimal and consistent with the surrounding code. Avoid broad refactors unless they are required by the task.
- Prefer existing helper functions and error types over introducing new patterns.
- When adding new features, update the relevant tests and keep evidence of verification in the final response.
- If a task touches CLI behavior, ensure the command-line interface and output semantics remain aligned with [src/main.rs](src/main.rs).

## Useful files to inspect first

- [Cargo.toml](Cargo.toml) for package metadata and dependencies
- [src/lib.rs](src/lib.rs) for the public surface and high-level library API
- [src/main.rs](src/main.rs) for CLI behavior and command definitions
- [src/validate.rs](src/validate.rs) for validation rules and image checks
- [src/results.rs](src/results.rs) for result types and output structure
