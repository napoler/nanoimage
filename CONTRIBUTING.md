# Contributing to NanoImage

Thank you for your interest in contributing!

## Prerequisites

- Rust 1.75+ (`rustup`)
- For GUI builds: X11 dev libraries (Linux) or native macOS/Windows

```bash
# Ubuntu/Debian
sudo apt install libxcb-render0-dev libxcb-shape0-dev libxcb-xfixes0-dev \
    libxdamage-dev libxrandr-dev libxi-dev libasound2-dev libgtk-3-dev
```

## Building

```bash
# All crates
cargo build --release

# Individual crates
cargo build --release -p nanoimage-cli
cargo build --release -p nanoimage-gui
cargo build --release -p nanoimage-core
```

## Testing

```bash
# All tests (73 tests across 10 suites)
cargo test --all --workspace

# Coverage per crate
cargo test -p nanoimage-core
cargo test -p nanoimage-cli
```

All tests must pass before submitting a PR.

## Benchmarks

```bash
cargo bench
```

Benchmarks are located in `crates/nanoimage-core/benches/`.

## Code Style

- Run `cargo fmt --all` before committing
- Run `cargo clippy --all --workspace -- -D warnings` — zero warnings required
- No `unsafe` blocks without justification and documentation
- All public APIs must have `///` doc comments

## Commit Messages

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add JPEG quality slider to GUI
fix: correct CJK width calculation in batch output
docs: update API reference for new config fields
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch from `main`
3. Make changes with tests
4. Ensure `cargo test --all` and `cargo clippy --all -- -D warnings` pass
5. Submit a PR with a clear description of changes

## Architecture

```
nanoimage/
├── crates/
│   ├── nanoimage-core/    # Core optimization engine
│   ├── nanoimage-cli/     # CLI tool
│   └── nanoimage-gui/     # Desktop GUI (egui)
├── tests/                  # Integration tests
└── benches/               # Performance benchmarks
```
