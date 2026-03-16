# AGENTS.md

Guidance for agentic coding assistants working in this repository.

## Scope
- Repo root: `D:\项目文件\code\galremote`
- Main focus: `src_assets/common/sunshine-control-panel`
- Stack: Tauri 2 + Rust backend + Vue 3/Vite frontend
- Platform assumptions: Windows-first

## Rule Files Status
- Cursor rules checked: `.cursor/rules/`, `.cursorrules` -> not found
- Copilot rules checked: `.github/copilot-instructions.md` -> not found
- If any of these appear later, treat them as highest-priority local guidance

## Working Directory Conventions
- Frontend/Tauri commands: run in `src_assets/common/sunshine-control-panel`
- Rust commands: run there with `--manifest-path src-tauri/Cargo.toml`, or in `src_assets/common/sunshine-control-panel/src-tauri`

## Install And Development
Run from `src_assets/common/sunshine-control-panel`:

```bash
npm install
npm run dev
npm run dev:renderer
npm run dev-webui
npm run dev:toolbar
```

Notes:
- `npm run dev`: full Tauri dev flow
- `npm run dev:renderer`: Vite renderer only
- `npm run dev-webui`: proxies to WebUI dev server at `https://localhost:3000`
- `npm run dev:toolbar`: build renderer + cargo build + launch toolbar mode

## Build Commands
Run from `src_assets/common/sunshine-control-panel`:

```bash
npm run build:renderer
npm run build:home
npm run build
npm run build:win
```

Notes:
- `npm run build`: full Tauri build
- `npm run build:win`: target `x86_64-pc-windows-msvc` with NSIS bundle

## Lint And Formatting
- This package currently has no JS/TS lint script in its own `package.json`
- Use Rust tooling:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets
```

- If you add JS/TS linting, add scripts to `package.json` and update this file

## Test Commands (Including Single-Test Runs)
Rust tests live under `src-tauri/src/**`.

Run all tests:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Run one test by exact test name:

```bash
cargo test --manifest-path src-tauri/Cargo.toml test_game_creation
```

Run one fully-qualified test path:

```bash
cargo test --manifest-path src-tauri/Cargo.toml galgame::game::tests::test_game_creation
```

Run tests for one module path/pattern:

```bash
cargo test --manifest-path src-tauri/Cargo.toml galgame::cloud::tests
```

Keep output visible while debugging:

```bash
cargo test --manifest-path src-tauri/Cargo.toml test_backend_type_name -- --nocapture
```

Important safety note:
- `src-tauri/src/galgame/oss_test.rs` is manual/credential-dependent test code with placeholder cloud keys
- Do not automate credential tests in CI without explicit secret handling

## Architecture Snapshot
- Frontend: Vue SFC pages/components under `src/renderer/**`
- Build: multi-entry Vite config at `src_assets/common/sunshine-control-panel/vite.config.js`
- Backend: Rust services and Tauri commands under `src-tauri/src/**`
- Command registration: centralized in `src-tauri/src/main.rs`

## Code Style Guidelines

### General
- Follow local style in the file being edited; avoid broad style churn
- Keep diffs narrow: do not mix unrelated refactors with behavior changes
- Preserve existing Chinese user-facing strings

### JavaScript / Vue
- Use ES modules (`import ... from ...`)
- Prefer `const`; use `let` only for reassignment
- Naming: `camelCase` for variables/functions, `PascalCase` for component filenames
- Keep `script setup` where already used
- Prefer async/await for Tauri invoke calls; guard with `try/catch`
- Log errors clearly with `console.error(...)`; do not silently swallow failures
- Avoid introducing new global-state patterns unless already established nearby

### Imports
- Group imports when practical:
  1. framework/library imports
  2. Tauri/API imports
  3. local project imports
- Keep ordering stable; avoid unrelated reorder-only diffs

### Formatting
- Frontend style is not fully uniform (some files use semicolons, some do not)
- Match punctuation and spacing in surrounding code
- Frontend default indentation: 2 spaces

### Rust
- Naming conventions:
  - `snake_case`: modules/functions
  - `CamelCase`: structs/enums/traits
  - `SCREAMING_SNAKE_CASE`: constants
- Prefer explicit `Result<T, String>` or project-consistent domain errors
- Tauri commands must use `#[tauri::command]`
- Return contextual errors via `map_err(...)` where possible
- Avoid `unwrap`/`expect` in production paths (acceptable in tests/prototypes)
- Use `log` macros (`info!`, `debug!`, `warn!`, `error!`) for operations and failures
- When adding/removing commands, keep `main.rs` invoke registration in sync

### Types And Data Contracts
- Prefer typed structs/enums over ad-hoc JSON blobs
- Use `serde::{Serialize, Deserialize}` for cross-boundary payloads
- Keep API payload fields stable unless change is explicitly required

### Error Handling Expectations
- Fail early with actionable messages
- Include context: what operation failed and why
- Preserve existing fallback behavior unless requirements change
- Never ignore errors silently; either log, return, or both

## Change Safety Rules
- Never commit secrets/tokens/real cloud credentials
- Be extra careful with Windows admin-elevation and system-command flows
- Keep transport/certificate policy changes minimal and explicit
- Confirm target paths and user intent before destructive file/sync actions

## Agent Workflow Recommendations
- Before editing: inspect nearby patterns and scripts
- After editing: run targeted validation for changed areas
- Prefer narrow checks during iteration; avoid unnecessary full rebuilds
- Before finalizing substantial changes:
  - `cargo test --manifest-path src-tauri/Cargo.toml` (or targeted subset)
  - `npm run build:renderer` when frontend is affected

## If You Add Tooling
- Update `package.json` scripts and this file together
- Document single-test invocation syntax for new test frameworks
- Keep this document command-accurate and concise
