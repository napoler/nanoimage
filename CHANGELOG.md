# CHANGELOG

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2026-07-18

### Added
- **CLI**: `compress` subcommand — single file JPEG/PNG/WebP optimization
- **CLI**: `batch` subcommand — directory batch processing with `--recursive`, `--format`, `--max-width`, `--max-height`, `--dry-run`, `--skip-failed`, `--only-unoptimized`
- **CLI**: `convert` subcommand — cross-format conversion (PNG↔JPEG↔WebP↔GIF)
- **CLI**: `settings` subcommand — configuration management (show/reset/modify)
- **GUI**: egui-based desktop application with drag-and-drop file support
- **GUI**: File panel with compression ratio color coding
- **GUI**: Settings panel with quality slider, compression mode, format selector
- **GUI**: Progress bar and log panel
- **GUI**: Completion notification dialog
- **GUI**: Config import/export
- **Core**: JPEG compression via `image` crate encoder
- **Core**: PNG optimization via `oxipng`
- **Core**: WebP encoding via `webp` crate
- **Core**: GIF and BMP passthrough
- **Core**: SVG passthrough
- **Core**: BatchProcessor with sync and async modes
- **Core**: Config serialization/deserialization (JSON)
- **Core**: Format detection and size formatting utilities
- **Tests**: 73 unit + integration + e2e tests across 10 test suites
- **Benchmarks**: Criterion benchmarks for JPEG/PNG/WebP compression
- **Packaging**: Debian .deb package build system
- **CI**: GitHub Actions workflow (build + test + clippy + fmt)

### Changed
- Refactored from single-crate to workspace with 3 crates (core, cli, gui)
- Replaced Python/Pillow with native Rust implementation
- Zero clippy warnings across all crates
- Fixed config persistence path inconsistency (CLI and GUI share `dirs::config_dir()`)
- Replaced `println!` in GUI with `tracing` macros

### Fixed
- CJK character width threshold in table output (0x2FF → 0x4E00)
- SVG validation now correctly skips XML declarations before `<svg>` check
- Prevented nested `optimized/` subdirectory creation on reprocessing
- File move now uses atomic rename with copy fallback for cross-filesystem
- Dropped unreliable mtime-based "already optimized" detection in batch processor
- Fixed production `unwrap()` on `Semaphore::acquire_owned().await` — now uses graceful skip
- Clippy `needless_borrows_for_generic_args` in GUI lib.rs
- Clippy `single_char_add_str` in CLI output.rs
- Clippy `cloned_ref_to_slice_refs` in processor_tests.rs
- Dead code warnings across all modules
- Missing input file existence check in compress command
- Unhandled Result from `save_config` in settings command
- Double `.metadata()` syscall in FileEntry::new()

### Documentation
- Complete README.md with quick start, features, installation, architecture
- CHANGELOG.md following Keep a Changelog format
- SPEC.md with full technical specification
- CONTRIBUTING.md for onboarding developers
- Inline documentation strings on all public APIs

### Security
- LICENSE: GPLv3 → MIT (matches README badge)
- Updated .gitignore to exclude Cargo.lock, .env files
