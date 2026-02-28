# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

- Here's my feature spec and test file: [path]. Implement the feature, then run the tests. If any fail, read the error output, fix the code, and re-run. Keep iterating until all tests pass. Do not ask me for input — just keep going until green. After all tests pass, commit with a descriptive message.
- Before proposing any fix, investigate these layers in parallel using subagents: 1) Configuration and environment variables, 2) Network/API connectivity and response formats, 3) Application logic and control flow from entry point to failure, 4) File permissions and process state. For each layer, report what you found. Then synthesize findings into a ranked list of likely root causes with evidence, and fix only the confirmed cause.

## Build & Test

```bash
cargo build                        # Build (also compiles .slint files via build.rs)
cargo run                          # Run the app
cargo test --lib                   # Run all unit tests
cargo test test_save_and_load      # Run a single test by name
```

Build compiles `ui/app.slint` as the root UI entry point via `slint-build` in `build.rs`.

## Architecture

Slint Chat is a desktop ChatGPT-style client built with **Slint UI + Rust backend**. It targets OpenAI-compatible APIs (configurable base URL).

### Two-Runtime Model

The app runs two runtimes concurrently:
- **Slint event loop** (main thread): UI rendering, callbacks, `Rc<RefCell<>>` state
- **Tokio runtime** (`Rc<Runtime>`): async HTTP streaming via `rt.spawn()`

Communication between them uses `tokio::sync::mpsc::unbounded_channel` for streaming tokens, and `slint::spawn_local()` to receive tokens back on the main thread and update UI.

### Layer Separation

```
UI (.slint files)
  ↕  callbacks + properties (<=> bindings)
main.rs (glue layer — all callback wiring, sync_*_to_ui helpers)
  ↕
lib.rs modules:
  ├── api/        — ApiClient (HTTP) + parse_sse_stream (SSE)
  ├── app_state   — AppState {topics, messages, active_topic_id}
  ├── models/     — Message, Topic, Assistant, Role
  ├── storage/    — Db (SQLite, 3 tables, direct rusqlite)
  ├── settings    — Settings (JSON file at config_dir/slint-chat/)
  └── markdown/   — pulldown-cmark → Vec<MarkdownBlock>
```

### Slint ↔ Rust Binding Pattern

- **Lists**: `Rc<VecModel<T>>` shared across closures; mutations (push/remove/set_row_data) auto-notify UI
- **Properties**: `<=>` two-way binding between AppWindow and child components
- **Callbacks**: Registered in main.rs via `ui.on_xxx()`, capture `ui.as_weak()` + `Rc` clones
- **Streaming update**: `update_last_assistant_message()` patches the last VecModel row in-place during SSE streaming

### Message Send Flow

1. User sends text → `on_send` callback
2. Create user Message → save to state + DB → push to VecModel (immediate UI)
3. Push empty assistant MessageItem placeholder to VecModel
4. `rt.spawn()`: ApiClient::chat_stream() → parse_sse_stream → send tokens via channel
5. `slint::spawn_local()`: recv tokens → accumulate → re-parse markdown → `set_row_data` on last row
6. Stream ends: save assistant Message to state + DB, set `sending = false`

### Data Persistence

- **Settings**: JSON at `dirs::config_dir()/slint-chat/settings.json`
- **Database**: SQLite at `dirs::config_dir()/slint-chat/chat.db`
- On macOS, `config_dir()` = `~/Library/Application Support/`

## Key Conventions

- UI strings are in Chinese (e.g., "新对话", "发送消息", "设置")
- CJK font bundled: `ui/fonts/NotoSansSC-subset.ttf`
- All entity IDs are UUID v4 strings
- Slint `TextInput` (basic element, no widget styling) preferred over `TextEdit` (std-widget with native borders) for custom-styled inputs
- Slint components that contain conditional `if` blocks should `inherit VerticalLayout` to ensure proper height calculation
