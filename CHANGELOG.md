# Changelog

## v0.2.6-beta.0 (2026-07-12)

### Added
- **Form URL-Encoded body type**: New `Form URL-Encoded` option in the Body tab for sending `application/x-www-form-urlencoded` requests (login forms, OAuth2 token exchange, etc.). Includes key-value editor with URL encoding.
- **WebSocket Enter-to-send**: Pressing Enter in the WebSocket input field now sends the message when connected (fixes dead `WsSendFromKeyboard` message).
- **Spinner on request loading**: Replaced plain "Loading..." text with animated spinner (`iced_aw::Spinner`) in both HTTP and GraphQL request views.
- Confirmation dialogs for history delete/clear (P1 #20).
- SSL verification disabled warning banner (P1 #6).
- OAuth2 token data sanitized before SQLite storage (P1 #7).
- WAL mode + foreign keys enabled for SQLite (P1 #11).

### Fixed
- **Script delay blocking UI**: `ScriptAction::Delay` now logs a warning instead of blocking the UI thread with `std::thread::sleep`.
- **Proxy auth**: Uses `Proxy::basic_auth()` instead of embedding credentials in URL string.
- **OAuth2 functions**: Now use shared `reqwest::Client` instead of creating new client per request.
- **Manual `impl Clone` for Message**: Replaced 50+ line manual implementation with `#[derive(Clone)]`.
- **Script derives**: `Script` and `RequestScripts` use `#[derive(Default)]` to fix clippy warnings.

### Tests
- 303 passing (4 new form-urlencoded tests added).
