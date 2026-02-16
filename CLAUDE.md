# Project Guidelines

## Rust Best Practices

- Run `cargo clippy` and fix warnings before committing
- Use `cargo fmt` before committing
- Prefer returning `Result` or `Option` over panicking
- Use `#[cfg(test)] mod tests` inline for unit tests
- Use the `tests/` directory for integration tests
- Use `indoc` for multi-line string literals in tests

## Commit Convention

Follow [Conventional Commits](https://www.conventionalcommits.org/):

```
<type>(<scope>): <description>
```

Types:

- `feat`: new feature
- `fix`: bug fix
- `refactor`: code change that neither fixes a bug nor adds a feature
- `test`: adding or updating tests
- `docs`: documentation changes
- `chore`: maintenance tasks (dependencies, CI, etc.)
