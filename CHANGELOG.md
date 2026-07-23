# Changelog

## v0.4.0 (2026-07-23)

### Added
- **Native OS menu bar** — File, Edit, View, Help menus via `muda` (macOS menu bar, Windows Win32 menu).
  - File: New Tab, Open Collection, Save, Import (cURL/Postman/OpenAPI), Export (Postman/HAR), Quit
  - Edit: Undo, Redo, Cut, Copy, Paste, Select All, Find
  - View: Toggle Sidebar, Toggle History, Toggle Collections, Toggle Dark Mode, New Window
  - Help: About Astraio
- **Form URL-Encoded body type** — `application/x-www-form-urlencoded` support with key-value editor.
- **WebSocket Enter-to-send** — Pressing Enter sends the message when connected.
- **Spinner on request loading** — Animated spinner (`iced_aw::Spinner`) during HTTP and GraphQL requests.
- Confirmation dialogs for history delete/clear.
- SSL verification disabled warning banner.
- OAuth2 token data sanitized before SQLite storage.
- WAL mode + foreign keys enabled for SQLite.

### Changed
- **Rebrand: AstraNova → Astraio** — Renamed across the entire codebase: struct names, database name, paths (`~/.astraio/`), CLI, HAR export creator, OAuth2 HTML titles, WiX installer.
- **Views refactor** — Split monolithic `views.rs` (1900 lines) into 10 focused modules (~200-420 lines each): `body_tab`, `auth_tab`, `settings_tab`, `cookies_tab`, `scripts_tab`, `response_area`, `snippets_panel`, `helpers`.
- **Toolbar spacing fix** — Request bar row kept inline in `view()` with exact `.spacing(10).padding(iced::Padding::from([16, 10]))` to preserve correct vertical spacing between toolbar and tabs.

### Fixed
- **Script delay blocking UI** — `ScriptAction::Delay` logs a warning instead of blocking the UI thread.
- **Proxy auth** — Uses `Proxy::basic_auth()` instead of embedding credentials in URL string.
- **OAuth2 functions** — Shared `reqwest::Client` instead of creating new client per request.
- **Menu timing on macOS** — Menu attached via `WindowOpened` subscription (after event loop starts), not during `AstraioApp::new()`.

### Tests
- 388 passing, 0 clippy warnings.
