# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

Tauri 2.0 desktop application with SvelteKit frontend and Rust backend. FCM Push Notification Tool for sending Firebase Cloud Messaging notifications.

## Principles

1. **Lint before commit**: Always run `cargo fmt && cargo clippy` before committing
2. **Type-safe errors**: Use `thiserror` for custom error types in Rust
3. **Keep CLAUDE.md updated**: Update this file when adding new features or patterns
4. **KISS**: Keep code simple, avoid unnecessary abstraction

## Build Commands

```bash
# Development
bun run tauri dev          # Run app in dev mode

# Frontend only
bun run dev                # Vite dev server
bun run check              # TypeScript type check

# Backend only
cd src-tauri
cargo check                # Check compilation
cargo fmt                  # Format code
cargo clippy               # Lint code

# Production
bun run tauri build        # Build release
```

## Project Structure

```
/src                          # Frontend (SvelteKit)
├── routes/
│   ├── +page.svelte         # Main page (발송/템플릿/히스토리 탭)
│   └── +layout.svelte       # Layout (일반/설정 탭 사이드바)
├── lib/
│   ├── bindings.ts          # Auto-generated (DO NOT EDIT)
│   └── components/
│       ├── Sidebar.svelte       # Navigation sidebar (일반/설정 탭)
│       ├── SendView.svelte      # FCM 발송 화면
│       ├── TemplateView.svelte  # 템플릿 관리 화면
│       ├── HistoryView.svelte   # 발송 히스토리 화면
│       └── SettingsView.svelte  # 설정 화면

/src-tauri                    # Backend (Rust)
├── src/
│   ├── lib.rs               # App initialization, command registration
│   ├── main.rs              # Entry point
│   ├── command.rs           # All Tauri commands
│   ├── modules/
│   │   ├── types.rs         # Shared types
│   │   └── logger.rs        # Logging utility
│   └── fcm/                 # FCM module
│       ├── mod.rs           # Module exports
│       ├── config.rs        # FcmConfig type
│       ├── auth.rs          # OAuth 2.0 + PKCE authentication
│       ├── pkce.rs          # PKCE utilities
│       ├── exchange.rs      # Token exchange via external API
│       ├── client.rs        # FCM HTTP API client
│       ├── message.rs       # FCM message types
│       ├── template.rs      # Template storage
│       └── history.rs       # History storage
└── Cargo.toml
```

## App Features

### 1. FCM 발송 (Send)
- **단일 디바이스**: 개별 FCM 토큰으로 푸시 발송
- **토픽**: 토픽 구독자 전체에 발송
- **일괄 발송**: 최대 500개 토큰에 동시 발송

### 2. 템플릿 (Template)
- 자주 사용하는 메시지 저장
- 템플릿 CRUD (생성/수정/삭제)
- 발송 화면에서 템플릿 불러오기

### 3. 히스토리 (History)
- 최근 100개 발송 기록 저장
- 성공/실패 상태, 상세 정보 확인
- 히스토리 일괄 삭제

### 4. 설정 (Settings)
- Firebase 프로젝트 ID 설정
- OAuth 2.0 클라이언트 ID 설정
- Google 계정 로그인/로그아웃

## TypeScript/Svelte Guidelines

### Imports
```typescript
import { onMount } from "svelte"
import { commands, type FcmConfig } from "$lib/bindings"
```

### Svelte 5 Runes
```typescript
let count = $state(0)
let doubled = $derived(count * 2)
```

### Result Type Handling
Commands return `Result<T, E>` type:
```typescript
const result = await commands.getConfig()
if (result.status === "ok") {
  config = result.data
} else {
  console.error(result.error)
}
```

### Formatting
- No semicolons
- Double quotes for strings
- 2 spaces indentation

## Rust Guidelines

### Tauri Commands
```rust
#[tauri::command]
#[specta::specta]
pub async fn my_command(app: AppHandle) -> Result<MyData, String> {
    // implementation
}
```

After creating a command:
1. Add to `collect_commands![]` in `lib.rs`
2. Run `bun run tauri dev` to regenerate TypeScript bindings

### Types for Frontend (specta)
```rust
#[derive(Debug, Clone, Serialize, Deserialize, specta::Type)]
#[serde(rename_all = "camelCase")]
pub struct MyData {
    pub user_name: String,
    pub created_at: DateTime<Utc>,
}
```

Note: Use `u32` instead of `usize` for specta compatibility.

### Tauri Store (Data Persistence)
```rust
use tauri_plugin_store::StoreExt;

let store = app.store("config.json")?;
let data: Option<MyData> = store.get("key").and_then(|v| serde_json::from_value(v).ok());
store.set("key", serde_json::to_value(&data)?);
store.save()?;
```

### Naming Conventions
| Type | Convention | Example |
|------|------------|---------|
| Files | snake_case | `my_module.rs` |
| Functions | snake_case | `get_user()` |
| Types/Structs | PascalCase | `AppState` |
| Constants | UPPER_SNAKE | `MAX_SIZE` |

## Data Storage

Uses Tauri Store plugin for persistent storage:

| File | Content |
|------|---------|
| `config.json` | App settings (OAuth, Firebase) |
| `token.json` | OAuth access/refresh tokens |
| `templates.json` | Message templates |
| `history.json` | Send history (max 100) |

Location: `~/.local/share/com.fcm-tool/` (varies by OS)

## OAuth 2.0 + PKCE Flow

1. Generate PKCE `code_verifier` and `code_challenge`
2. Build auth URL with `code_challenge`
3. Open browser for Google login
4. Start local callback server (port 8080)
5. Receive authorization code
6. Exchange code via external API (`exchange_code_url`)
7. Store tokens in Tauri Store

## Before Committing

Husky pre-commit hook automatically runs:
- Git user.name/email verification
- `cargo fmt --check` and `cargo clippy`
- `bun run check` for TypeScript validation

Manual checks (if needed):
```bash
cd src-tauri && cargo fmt && cargo clippy
bun run check
```

## Release Process

Releases are automated via GitHub Actions when you push a tag:

```bash
git tag v1.0.0
git push origin v1.0.0
```

This triggers `.github/workflows/release.yml` which:
1. Builds for Windows, macOS (Universal), Linux
2. Creates draft release with platform binaries

## Common Tasks

### Adding a New Command
1. Define in `src-tauri/src/command.rs` with `#[tauri::command]` + `#[specta::specta]`
2. Add to `collect_commands![]` in `lib.rs`
3. Run `bun run tauri dev` to regenerate bindings
4. Use via `commands.myCommand()` in frontend

### Modifying FCM Types
1. Update struct in `src-tauri/src/fcm/*.rs`
2. Ensure `#[serde(rename_all = "camelCase")]` for frontend compatibility
3. Run app to regenerate bindings

## Key Dependencies

### Rust
- `tauri` 2.0 - App framework
- `tauri-plugin-store` - Data persistence
- `tauri-specta` - TypeScript bindings generation
- `reqwest` - HTTP client for FCM API
- `chrono` - DateTime handling

### Frontend
- `svelte` 5.x - UI framework (with runes)
- `@skeletonlabs/skeleton-svelte` - UI components
- `tailwindcss` 4.x - CSS framework
