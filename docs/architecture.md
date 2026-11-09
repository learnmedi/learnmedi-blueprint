# Architecture: learnmedi_blueprint

The shared plan used by engine and site. **Definitions and pure validation only**: no DB, HTTP, rendering or AI.

- `blocks.rs`: the block registry (every block type, its fields and limits, its group: Presentation / Core / Excerpt), `validate_props`, and `validate_url` (allow-list).
- `tabs.rs`: `TabKind` (Free, Blog, Courses, Collection, Link), each kind's core block and allowed groups, collection templates, and `validate_tab_blocks`.
- `theme.rs`: theme schema, validation and the fallback look. The catalog of looks is not here.
- `themes.rs`: rules for the theme catalog (a look plus optional starter pages built from shared slots: opener, about, story, work, posts, products, community, contact): validating an entry, building starter pages, saving a live page as a starter (`strip_page`) and porting a page into another theme's layout (`port_page`). The catalog itself lives in the engine database (`themes` collection; built-ins in `learnmedi_engine/crates/engine/themes/`).
- `models.rs`: shared models (User, SiteRecord, roles…).
- `internal.rs`: request and response types for the site → engine internal API, plus `SiteSettings` (defaults let a site start while the engine is down).
- `util.rs`, `error.rs`: helpers and `AppError`.

See `learnmedi_site/docs/architecture.md` for how a page is served and how the AI edits sites.
