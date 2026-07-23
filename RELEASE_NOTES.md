# v0.4.0 – 2026-07-23

## New features
- **Native OS menu bar** — Full File/Edit/View/Help menus via `muda` crate (macOS menu bar, Windows Win32 menu). Keyboard shortcuts shown inline (⌘T, ⌘S, ⌘F, etc.).
- **Form URL-Encoded body type** — `application/x-www-form-urlencoded` support with key-value editor for login forms and OAuth2 token exchange.
- **WebSocket Enter-to-send** — Pressing Enter sends the message when connected.
- **Spinner on request loading** — Animated spinner during HTTP and GraphQL requests.

## Breaking changes
- **Rebrand: AstraNova → Astraio** — Database renamed from `astranova.db` to `astraio.db`. Data paths changed from `~/.astranova/` to `~/.astraio/`. Users migrating from v0.2.x should move their database file.

## Known limitations
- Native menu bar works on macOS and Windows. Linux uses in-app fallback (no SO menu bar).
- Windows menu accelerators (Ctrl+S, etc.) require `TranslateAcceleratorW` which iced doesn't expose — shortcuts work via in-app keyboard subscriptions, not via the native menu.

---

*Generated on 2026-07-23.*
