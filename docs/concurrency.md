# Concurrency and lock ordering (desktop shell)

The Tauri host keeps per-window state in `WindowSession` (`window_session.rs`). Most UI commands run on the **main thread**, but nested `invoke` calls (for example `browser-state-updated` → `get_browser_state` → `snapshot`) can re-enter the same thread. **Never hold `tabs` while calling `snapshot`, `update_content_webview`, or another command that locks `tabs` again.**

## Lock order (high → low)

1. `AppState::sessions`
2. Per-window `WindowSession::tabs` (`TabsMutex` in debug builds detects re-entrancy)
3. Other per-window mutexes (`close_undo`, `split_view`, `undo_stacks`, `file_compare`, …)
4. `AppState::command_manager`, `registry`, `keybindings`
5. Global `live_by_window()` map (`live_host.rs`) — **drop before** calling `unibrow_live_status` (status re-locks the map)

Acquire in that order when one call needs multiple locks. **Do not** hold `tabs` while acquiring `command_manager` ( `refresh_command_manager_context` locks `tabs` then `command_manager`, then releases both before returning).

## Safe pattern

```rust
let loc = {
    let mut tabs = sess.tabs.lock().map_err(|e| e.to_string())?;
    tabs.navigate_active(loc.clone(), is_dir, doc);
    loc
};
update_content_webview(&app, window_label, &loc)?;
snapshot(&state, window_label)?;
```

Clone tab locations or DTO inputs inside the guard, drop the guard, then call helpers that may re-lock.

## Debug builds

- `TabsMutex` panics on **re-entrant** `tabs.lock()` on the same thread.
- `snapshot()` calls `tabs_mutex::assert_tabs_lock_free()` so a held guard fails fast in `cargo test` / debug desktop runs.

## Self-test

In-app `--self-test` sets `UNIBROWSER_SELF_TEST=1` and skips `browser-state-updated` emits while opening files so the UI thread does not nest `get_browser_state` inside `open_path`.
