# Rust Maintainability Guidelines

This document outlines best practices for writing maintainable Rust code in this project.

---

## Project Structure

### Directory Layout

```
project/
├── Cargo.toml / Cargo.lock
├── src/
│   ├── main.rs          # Binary entry point
│   ├── lib.rs           # Library root
│   ├── config.rs        # Configuration
│   ├── error.rs         # Error types
│   ├── models/          # Data structures
│   │   ├── mod.rs
│   │   └── *.rs
│   ├── services/        # Business logic
│   │   ├── mod.rs
│   │   └── *.rs
│   └── utils/           # Helpers
│       ├── mod.rs
│       └── *.rs
├── tests/               # Integration tests
├── benches/             # Benchmarks
├── examples/            # Usage examples
└── docs/                # Documentation
```

### Module Organization

- Export public modules from `lib.rs` with `pub mod`
- Re-export key items with `pub use` for a clean public API
- Each module should have a `mod.rs` that explicitly lists exports

---

## Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Variables | `snake_case` | `user_count` |
| Functions | `snake_case` | `calculate_total()` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_CONNECTIONS` |
| Structs/Enums/Traits | `PascalCase` | `UserProfile`, `PaymentStatus` |
| Type Aliases | `PascalCase` | `type UserId = u64` |
| Crates | `kebab-case` | `my-crate` |
| Modules | `snake_case` | `mod user_auth` |

---

## Code Organization

### Function Design

- **Single responsibility**: Each function should do one thing well
- Compose small, focused functions rather than large multi-purpose functions

```rust
// Good: Composed small functions
fn process_user_data(user: &mut User, data: &UserData) -> Result<(), Error> {
    validate_user_data(data)?;
    update_user(user, data);
    save_user(user)?;
    Ok(())
}
```

### Type-Driven Design

Use custom types to enforce invariants at compile time:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct AccountId(u64);

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
struct Money(f64);

impl Money {
    fn new(amount: f64) -> Result<Self, Error> {
        if amount <= 0.0 { return Err(Error::InvalidAmount); }
        Ok(Money(amount))
    }
}
```

### Public vs Private Fields

**Do NOT use getter/setter patterns** (e.g., `get_field()`, `set_field()`). Rust provides direct field access.

| Case | Approach |
|------|----------|
| Simple data container (DTO, config) | Use `pub` fields directly |
| Invariant validation required | Use `private` fields + `new()` constructor |

```rust
// Good: Simple data - public fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStats {
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

// Good: Enforce invariant - private + constructor
#[derive(Debug, Clone)]
pub struct Percentage(u8);

impl Percentage {
    pub fn new(value: u8) -> Result<Self, Error> {
        if value > 100 { return Err(Error::InvalidPercentage); }
        Ok(Percentage(value))
    }
}

// Avoid: Getter/setter anti-pattern
// ❌ fn get_value(&self) -> u8 { self.value }
// ❌ fn set_value(&mut self, v: u8) { self.value = v }
```

### Separation of Concerns

- Separate pure business logic from I/O (database, network, filesystem)
- Business logic should be testable without actual I/O
- Use traits for dependency injection

---

## Error Handling

### Custom Error Types

Use `thiserror` for library errors and `anyhow` for applications:

```rust
#[derive(Error, Debug)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
    
    #[error("Validation error: {0}")]
    Validation(String),
    
    #[error("Resource not found: {0}")]
    NotFound(String),
}

pub type Result<T> = std::result::Result<T, AppError>;
```

### Error Context

Add context to errors to aid debugging:

```rust
let content = std::fs::read_to_string(path)
    .with_context(|| format!("Failed to read config file: {}", path))?;
```

### Patterns

- Use early returns for error conditions
- Collect multiple validation errors into a `Vec`
- Use `and_then()` for railway-oriented programming

---

## Documentation

### Levels

| Level | Location | Purpose |
|-------|----------|---------|
| Crate | `lib.rs` | Overview, features, examples |
| Module | `mod.rs` | Purpose, design decisions |
| Function | Each pub fn | Arguments, returns, errors, examples |

### Syntax

- `//!` for crate/module-level internal docs
- `///` for public API external docs
- Include: Description, Arguments, Returns, Examples, Errors, Safety (if applicable)

---

## Testing

### Unit Tests

Place in same file with `#[cfg(test)]` module:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_function() {
        assert_eq!(function(2, 3), 5);
    }
}
```

### Integration Tests

Place in `tests/` directory.

### Property-Based Testing

Use `proptest` for randomized testing of properties.

---

## Dependencies

### Version Strategy

| Project Type | Strategy |
|--------------|----------|
| Application | Semver range: `1.0.164` (allows compatible updates) |
| Library | Range: `>=1.0.150, <2.0.0` |

**Note**: Avoid exact pinning (`=1.0.164`) for applications as it prevents security updates. Use semver ranges to allow compatible version updates while maintaining stability.

### Minimization

- Use `default-features = false` to disable unused features
- Enable only required features explicitly
- Run `cargo audit` regularly for vulnerability checks
- Use `cargo outdated` to find updates

---

## Code Quality Tools

### Required Commands

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Lint
cargo clippy

# Type check
cargo check
```

### Recommended Workflow

Run these commands before every commit:
1. `cargo fmt`
2. `cargo clippy`
3. `cargo check`

---

## Performance

### Guidelines

- **Avoid premature optimization**: Write clear code first
- **Profile before optimizing**: Use `criterion` for benchmarks
- **Choose right data structure**:
  - `Vec` — ordered, indexed access
  - `HashMap` — key-value lookups
  - `HashSet` — unique values, membership
  - `BTreeMap` — sorted keys

---

## Related

- [@docs/policy/coding-standards.md](coding-standards.md) — Coding standards
- [@docs/policy/naming-conventions.md](naming-conventions.md) — Naming rules
