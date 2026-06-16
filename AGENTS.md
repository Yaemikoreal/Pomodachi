# Repository Guidelines

## Project Structure & Module Organization

```
toumato/
├── src/                  # React frontend (TypeScript)
│   ├── components/       # 7 components: SpinePet, StatusPanel, Sidebar, ChatPanel, etc.
│   ├── hooks/            # 8 hooks: useTimer, usePet, useChat, useTasks, usePomodoro, etc.
│   │   └── __tests__/    # Hook unit tests (Vitest)
│   ├── test-utils/       # Tauri IPC mock utilities
│   ├── data/             # Static data definitions (skins.ts)
│   └── types/            # TypeScript type definitions (model.ts)
├── src-tauri/            # Rust backend (Tauri v2)
│   └── src/
│       ├── lib.rs        # 43 IPC commands + event listeners + system tray
│       ├── timer.rs      # Pomodoro state machine (Tokio-driven async timer)
│       ├── pet.rs        # Pet mood state machine (6 moods + skin switching)
│       ├── db.rs         # SQLite database (9 tables, migration support)
│       ├── ai.rs         # Claude CLI subprocess communication
│       ├── monitor.rs    # Windows foreground process monitor
│       ├── task.rs       # Task CRUD wrapper
│       └── notification.rs  # Windows MessageBox notifications
├── public/spines/        # Spine 4.1 skeleton animation assets
└── docs/                 # Design docs and troubleshooting guides
```

Frontend hooks encapsulate all Tauri IPC calls; components consume hooks only. Rust modules are independent, coordinated through `AppState` which injects all managers into commands.

## Build, Test, and Development Commands

| Command | Purpose |
|---------|---------|
| `pnpm tauri dev` | Start full dev environment (frontend + Rust backend) |
| `pnpm dev` | Start frontend-only Vite dev server (no Tauri) |
| `pnpm tauri build` | Build production Tauri app |
| `pnpm test` | Run all Vitest frontend tests |
| `pnpm tsc --noEmit` | TypeScript type check |
| `cd src-tauri && cargo test` | Run all Rust unit tests |
| `cd src-tauri && cargo check` | Check Rust code without compiling |
| `cd src-tauri && cargo fmt` | Format Rust source code |

## Coding Style & Naming Conventions

- **TypeScript/React**: Strict mode (`strict: true`), `noUnusedLocals` and `noUnusedParameters` enabled. Use React 19 patterns with hooks and functional components.
- **Rust**: Follow standard `cargo fmt` formatting. IPC structs use `#[serde(rename_all = "camelCase")]` for automatic Rust-to-JS field conversion.
- **Naming**: Rust modules are single-word lowercase (`timer.rs`, `pet.rs`). Frontend components are PascalCase (`SpinePet.tsx`), hooks use camelCase with `use` prefix (`useTimer.ts`). Test files match their source file with `.test.ts` suffix.
- **Styling**: Inline CSS with Framer Motion for animations. No TailwindCSS.

## Testing Guidelines

- **Frontend**: Vitest 4 with `jsdom` environment, `globals: true`. Test files live in `src/hooks/__tests__/` and match the pattern `*.test.ts`.
- **Rust backend**: Standard `cargo test` with `#[cfg(test)]` modules. Database tests use `new_in_memory()` to avoid touching real SQLite files.
- **Mock pattern**: Frontend tests mock Tauri IPC via `src/test-utils/mockTauri.ts`, which exposes `__setInvokeMock`, `__emitEvent`, and `__resetMocks`. Tests follow: register mock, call hook, assert state change.
- **Strategy**: Write tests first (mock frontend or in-memory DB backend), confirm failure, implement, confirm pass.

## Commit & Pull Request Guidelines

- **Commit format**: Use Chinese, structured as `[type]：Concise summary`. Types from history: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `style`, `chore`.
- **PR requirements**: Clear description linking to the issue (`Closes #123`), screenshots if UI changes, verify `pnpm tauri dev` runs correctly, self-review checklist. Keep PRs focused on a single concern.

## Architecture Overview

This is a transparent, always-on-top desktop pet built with Tauri v2. The Rust backend owns all state and drives a Pomodoro timer asynchronously via Tokio. It pushes events to the React frontend through Tauri's event system (`timer-tick`, `timer-complete`, `distraction-detected`, `achievement-unlocked`, `pet-skin-changed`), and the frontend invokes commands through hooks that wrap `@tauri-apps/api/core`.

Spine 4.1 skeleton animations render via PixiJS Legacy 7.4 with `@pixi-spine/all-4.1`. The pet has 6 emotional states mapped to face animations on a separate animation track. Skin assets live in `public/spines/` per skin directory.

AI integration spawns a `claude` CLI child process, streaming or blocking. The chat system maintains session continuity through a JSON response `session_id` field.
