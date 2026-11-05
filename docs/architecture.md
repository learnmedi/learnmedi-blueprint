# Architecture: learnmedi_blueprint

The shared plan used by engine and site. **Definitions and pure validation only**: no DB, HTTP, rendering or AI.

- `blocks.rs`: the block registry (every block type, its fields and limits, its group: Presentation / Core / Excerpt), `validate_props`, and `validate_url` (allow-list).
- `tabs.rs`: `TabKind` (Free, Blog, Courses, Collection, Link), each kind's core block and allowed groups, collection templates, and `validate_tab_blocks`.
- `theme.rs`: theme schema and presets.
- `models.rs`: shared models (User, SiteRecord, roles…).
- `internal.rs`: request and response types for the site → engine internal API, plus `SiteSettings` (defaults let a site start while the engine is down).
- `util.rs`, `error.rs`: helpers and `AppError`.

See `learnmedi_site/docs/architecture.md` for how a page is served and how the AI edits sites.
