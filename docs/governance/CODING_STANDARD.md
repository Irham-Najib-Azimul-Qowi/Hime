# Coding Standard
**Project:** Hime Assistant  
**Version:** 1.0  

---

## 1. General Principles

- **Clarity over cleverness.** Code should be understandable by a new team member.
- **Consistency over preference.** Follow the project conventions even if you prefer alternatives.
- **Explicit over implicit.** Make behavior visible and predictable.

---

## 2. Language-Specific Standards

### Rust (Hime Desktop / Tauri)

- Follow the [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/).
- Use `clippy` with default lints enabled.
- Use `rustfmt` for formatting.
- Prefer `Result<T, E>` over panicking. Reserve `unwrap()` for truly impossible cases with a comment explaining why.
- Minimize `unsafe` blocks. Document safety invariants when used.
- Use `#[must_use]` on functions whose return values should not be ignored.

### TypeScript / JavaScript (Hime Desktop WebView)

- Use TypeScript with strict mode enabled.
- Use ESLint with recommended rules.
- Use Prettier for formatting.
- Prefer `const` over `let`. Never use `var`.
- Prefer named exports over default exports.
- Handle async errors with try/catch, not unhandled promise rejections.

### Python (Hermes Agent integration)

- Follow PEP 8.
- Use type hints for function signatures.
- Use `ruff` for linting and formatting.
- Prefer `pathlib.Path` over string path manipulation.
- Use `dataclasses` or `pydantic` for structured data.

### Kotlin (Hime Mobile)

- Follow [Kotlin Coding Conventions](https://kotlinlang.org/docs/coding-conventions.html).
- Use Jetpack Compose for UI.
- Follow MVVM/MVI architecture patterns.

---

## 3. Documentation Standards

- Every public function, struct, class, and module must have a doc comment.
- Doc comments should describe **what** and **why**, not **how** (the code shows how).
- Use markdown in doc comments for code references and examples.
- Keep README files up to date with any structural changes.

---

## 4. Error Handling

- Always handle errors explicitly.
- Never silently swallow exceptions.
- Log errors with sufficient context (what operation, what input, what went wrong).
- Distinguish between recoverable errors (retry, fallback) and fatal errors (crash with message).
- User-facing error messages should be helpful, not stack traces.

---

## 5. Naming Conventions

| Context | Convention | Example |
|:---|:---|:---|
| Rust types/structs | PascalCase | `DynamicIsland`, `AgentConfig` |
| Rust functions/variables | snake_case | `create_window`, `agent_status` |
| TypeScript/JS types | PascalCase | `IslandProps`, `AgentState` |
| TypeScript/JS functions/variables | camelCase | `createWindow`, `agentStatus` |
| CSS classes | kebab-case | `island-container`, `status-badge` |
| File names | kebab-case | `dynamic-island.rs`, `agent-config.ts` |
| Constants | SCREAMING_SNAKE_CASE | `MAX_RETRY_COUNT`, `DEFAULT_PORT` |
| Database tables | snake_case | `agent_sessions`, `tool_usage` |

