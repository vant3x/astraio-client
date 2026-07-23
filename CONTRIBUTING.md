# Contributing to Astraio Client

Thanks for your interest in contributing! This guide will help you get started.

## Development Setup

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) 1.75+
- Platform-specific dependencies (Linux only):
  ```bash
  sudo apt install libxkbcommon-dev libwayland-dev libxrandr-dev libxcursor-dev libxi-dev libxinerama-dev libglib2.0-dev libgtk-3-dev
  ```

### Build & Run

```bash
git clone https://github.com/vant3x/astraio-client-rust.git
cd astraio-client-rust
cargo run           # Debug build
cargo run --release # Release build
```

### Run Tests

```bash
cargo test
```

## Pull Request Checklist

Before submitting a PR, make sure:

- [ ] `cargo fmt` — code is formatted
- [ ] `cargo clippy -- -D warnings` — no clippy warnings
- [ ] `cargo test` — all tests pass
- [ ] New features have tests where applicable
- [ ] Commit messages are clear and concise

## Code Style

- Follow existing patterns in the codebase
- Keep view functions focused (~200-400 lines per file)
- Use `Message` variants for UI events, handlers in `src/ui/handlers/`
- Prefer `#[derive(Default)]` over manual `impl Default`
- Never commit secrets, API keys, or credentials

## Architecture

```
src/
├── ui/          # Iced GUI (app.rs, views/, handlers/, components/)
├── http_client/ # HTTP client, request/response models
├── protocols/   # WebSocket, GraphQL, script engine
├── data/        # Data models (auth, OAuth2)
├── persistence/ # SQLite database
├── services/    # Business logic
├── import/      # cURL, Postman import
├── export/      # Postman, HAR export
└── openapi/     # OpenAPI/Swagger import
```

## Reporting Issues

- Use [GitHub Issues](https://github.com/vant3x/astraio-client-rust/issues)
- Include OS, Rust version, and steps to reproduce
- For security issues, email security@astraio.dev directly

## License

By contributing, you agree that your contributions will be licensed under the [MIT License](LICENSE).
