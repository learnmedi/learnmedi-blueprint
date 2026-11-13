# Architecture: learnmedi_blueprint

The shared plan used by engine and site. **Definitions and pure validation only**: no DB, HTTP, rendering or AI.

- `blocks.rs`: the block registry (every block type, its fields and limits, its group: Presentation / Core / Excerpt), `validate_props`, and `validate_url` (allow-list).
- `features.rs` + `ui/features/*.yaml`: **the feature registry**, one YAML file per feature (and `00-core.yaml` for the website itself). A feature declares its plan metadata (key, label, tier, locked…), its visitor surfaces (a Page = tab kind with one core block, Blocks, Widgets), routes, reserved slugs, AI tools (dangerous ones marked) and its AI prompt text. A file with `enabled_by` is a tool group (course and quiz tools, on with Classes or Self-paced). It answers: which feature a tool or route needs (`tool_gate`, `route_gate`), `dangerous_tools`, `reserved_slugs`, which feature owns a block/widget/tab kind, and the AI prompt: `prompt_for(on)` expands the core template's `{prompt:KEY#part}` lines for the features that are on for the AI; the site then `fill`s the runtime placeholders (`{goal_block}`, `{overview}`, `{catalog}`…) in one pass. `prompt_parts()` feeds the engine's prompt viewer. Adding a feature = one YAML file here (placed in the core template) + its logic in the site.
- `widgets.rs`: site-wide elements on every public page (`fortune_button`, `newsletter_prompt`); owners switch them on, the admin can switch them off.
- `plans.rs`: tiers and the plan board (`PlanConfig`). `CATALOG` is built from the feature YAML. Each `FeatureSetting` has an `ai` switch (missing in old boards = on); `blocks` / `widgets` hold what the admin switched off. `resolve` / `resolve_ai` give a site's effective features.
- `scholar.rs`: pure rules for the scholar features: DOI / ORCID iD checks, Crossref / OpenAlex / ORCID parsers, h-index, the HĐGSNN points self-assessment (QĐ 37/2018, 25/2020: PGS 10, GS 20 points; main author 1/3 + equal share of 2/3) and `ScholarSettings` (ORCID, field, target, public badge, CV fields).
- `tabs.rs`: `TabKind` (Free, Blog, Courses, Collection, Link, Products, Community, Surveys, Associates), each kind's core block and allowed groups, collection templates, and `validate_tab_blocks`. Kinds that own a feature's data are `single` (one per site and language).
- `theme.rs`: theme schema, validation and the fallback look. The catalog of looks is not here.
- `themes.rs`: rules for the theme catalog (a look plus optional starter pages built from shared slots: opener, about, story, work, posts, products, community, contact): validating an entry, building starter pages, saving a live page as a starter (`strip_page`) and porting a page into another theme's layout (`port_page`). The catalog itself lives in the engine database (`themes` collection; built-ins in `learnmedi_engine/crates/engine/themes/`).
- `models.rs`: shared models (User with `default_site_id`, SiteRecord with the effective `features`, `ai_features`, `disabled_blocks`, `disabled_widgets` sent by the engine, roles…).
- `internal.rs`: request and response types for the site → engine internal API, plus `SiteSettings` (defaults let a site start while the engine is down).
- `util.rs`, `error.rs`: helpers and `AppError`.

See `learnmedi_site/docs/architecture.md` for how a page is served and how the AI edits sites.
