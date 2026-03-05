# Coding Standards (Rust)

## Error Handling

- Use `anyhow` for application errors, `thiserror` for library-like reusable components
- Use `?` operator in production — avoid `.unwrap()` unless justified
- Add context to errors with `.context()` to show where the error occurred

## Dependencies

- `thiserror` — custom error types
- `anyhow` — application-wide error handling
- `tracing` — structured logging
- `serde` — data serialization (JSON/TOML)
- `clap` — CLI argument parsing
- `ratatui` — TUI components
- `chrono` — date/time handling

## Derive Traits

Always derive on data structs:

- `Debug`, `Clone` — always
- `Serialize`, `Deserialize` — for any data that leaves the process
- `PartialEq`, `Eq`, `Hash` — for enums used as map keys or comparison
- `Default` — for structs with sensible defaults

## Logging

- Use `tracing` for all logging (not `log` crate)
- Include contextual information (server name, user action, etc.)
