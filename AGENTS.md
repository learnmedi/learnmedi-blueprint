# AGENTS.md — rules for anyone (human or AI) changing Learnmedi

## 🚫 Hard rules

1. **Never store files in MongoDB.** No image/video/PDF/homework bytes, no base64
   strings of files, no GridFS, no `bson::Binary`. Files go to **Google Cloud
   Storage** (`the site storage module`); Mongo keeps only `Asset` metadata
   (object key, type, size). The test `no_files_in_mongo_tests.rs` fails the
   build if this is broken. The bucket name comes from `.env` (`STORAGE_BUCKET`).
   The same holds for git: photos, images, video, PDFs and other media are never
   committed to a repo or embedded in code or scripts. Upload them to storage and
   keep only the link (object key or URL). A script that needs local photos reads
   them from a folder outside the repo (like the demo's `LM_REF_DIR`).
2. **The AI edits the site only through tools → `ops::apply`.** It never writes
   code, HTML, CSS or files. Any new capability the AI needs = a new validated op
   in `crates/site/src/ops.rs` + a tool definition in `crates/site/src/ai/tools.rs`
   (the test `every_write_op_has_a_tool` checks they stay in sync). Every op
   validates input, writes a `Revision` (undo) and returns client actions.
3. **Everything user/AI-provided is escaped on render.** Markdown is rendered
   with raw HTML disabled; URLs are allow-listed (`blocks::validate_url`). Do not
   add a "raw HTML" block.
4. **Secrets stay in `.env`** (never committed). `.env.prod` and service-account
   JSON files are git-ignored.
5. **Only site → engine internal API calls.** Use a Bearer token and the private listener.
   The engine does not call the site's internal API. The site can scale horizontally.
6. **Email only with consent.** Newsletters go only to `confirmed` subscribers
   (double opt-in or a Google sign-in with that exact address), always with a
   personal unsubscribe link, and count against the site's daily quota
   (`newsletter::reserve`). Real sends from the AI need `confirm=true`.
7. **Themes come from real, human-designed sites.** A theme (home page and starter
   pages) follows one reference site for at least 68%, or two reference sites
   mixed for at least 90% together. Take layout and composition only, never their text, images or
   logos, and name the source in a comment at the top of the theme YAML. The
   reference list and how to measure it are in
   `learnmedi_engine/docs/theme-references.md`. A theme made up without a
   reference is not accepted.


## Repository scope

Shared types and validation. Keep database queries, network calls, rendering and AI in the services.

## Feature architecture (built 2026-11-14, see "Status" below)

Decided 2026-11-13. Applies to blueprint, engine and site. Read it before adding or
changing a feature, a tab kind, a block, a widget, an AI tool or AI prompt text.

Detailed step-by-step plan (files, schema, tool map, tests, migration, release): `learnmedi_blueprint/docs/feature-architecture-plan.md`. Start any session on this work by reading it.

### One registry for every feature
Every feature (Blog, Products, Community, Surveys, Associates, Fortune, Personality,
Teaching, Newsletter, Assets, …) and the **core** (pages, blocks, theme, publish, undo)
is declared once in `learnmedi_blueprint/ui/features/*.yaml`. It extends the existing
`plans::CATALOG` (`FeatureDef`); it does not replace it. A feature declares:
- `key`, `label`, `description`, `workspace`, `default_tier`, `default_enabled`, `locked`
- where it appears to visitors, in three surface kinds (a feature may have several, or none):
  - **Page** — a tab kind with exactly one core block (`tab:` core block, allowed block groups, default slug)
  - **Block** — blocks placed on any page that read the feature's data (`blocks:`)
  - **Widget** — site-wide element on every public page, switched on by the owner (`widgets:`; e.g. the fortune-stick button, the newsletter prompt)
- where it appears to staff (`workspace` / owner overview sections) and its `routes` (public and staff)
- its AI `tools` (each marked `dangerous: true|false`) and its AI `prompt` text
- its reserved slugs

Features visitors never see (Personality, Question bank, Classes, …) are features too:
they declare routes, tools and prompt, and no Page/Block/Widget.

### All AI prompt text lives in blueprint
The core prompt (role, how to work, find → show → confirm → act, languages) and every
feature's prompt are blueprint text. The site only assembles them and injects runtime
context (the site's pages and block ids, today's date, the selected page/block, theme
schema, block catalog). No prompt prose is written in site code.

### Who does what
- **learnmedi_blueprint**: the skeleton — feature YAML, tab kinds, block registry, widget
  list, tool → feature map, dangerous-tool list, reserved slugs, prompt text, plan/feature
  settings types. Definitions and pure validation only (no DB, HTTP, rendering, AI calls).
- **learnmedi_site**: the logic — rendering, data, running tools. Reads routes, tools,
  dangerous tools, reserved slugs and prompt from blueprint; never keeps its own copy.
  Enforces switches at run time: a disabled feature/Page/Block/Widget does not render and
  is not offered in Studio; a tool whose feature is off for AI is neither sent to the
  model nor executed. **The master token may run every op and tool.**
- **learnmedi_engine**: the admin configuration board (`/admin`): switch each feature,
  Page kind, Block and Widget on/off (per tier and per-site overrides), switch AI access
  per item, and **view the prompt text** of each item (read directly from blueprint).

### Merging fixed pages into tabs
`/products`, `/community`, `/research/surveys`, `/associates` become tab kinds with a core
block (like Blog). Their data stays in its own collections. A one-off migration adds, per
site that uses them, one tab with the old slug so old links keep working; it is idempotent
(skips sites that already have that tab kind) and is handed to the maintainer to run on
Mongo. `/learn` is a learner workspace and stays a fixed route.

### Owner overview and dashboard shortcut
- `{site_url}/overview` — **owner only** (not editors). It is the hub of site management.
  Its main panel shows: stat cards (publications, Q1/Q2 count, citations, h-index, views
  in 30 days), HĐGSNN points progress against the PGS/GS standard, publications found via
  ORCID awaiting review, and a to-do list (missing photo, papers without DOI, no English
  version…). Its own left sidebar is grouped; new features add a line here, never to a top bar:
  - Tổng quan
  - Hồ sơ khoa học: Công bố · Chỉ số · Điểm HĐGSNN · Lý lịch khoa học
  - Website: Studio ↗ · Nội dung ↗ · Lượt xem
  - Giảng dạy ↗ (only when the tier has Teaching) · Cài đặt ↗
  (↗ = opens the existing workspace page.) The menu list is defined once and also feeds
  the "Chuyển ▾" switcher.
- In the other workspaces (Studio, Content, Teaching, Settings) the top bar no longer lists
  workspaces; it shows a **"Quản lý trang"** link to `/overview` plus the switcher.
- `overview` is a reserved slug.
- `engine.learnmedi.com/dashboard?site=N`: `0` = the user's default site, `N` = their
  N-th site (owner or staff, ordered by `ts_created` ascending, the same order and `#N`
  numbers shown on `/account`). Valid → 302 to `{site_public_url}/overview`. Not signed in →
  `/login?next=…`. Invalid, out of range or no site → `/account#sites`. No `site` param →
  the current redirect to `/account#sites`. The default is stored as a site id
  (`User.default_site_id`), never as a position; if it is gone, use the first site.
  Emails always link `?site=0`.
- Publications and achievements are data (collection items), shown by blocks. Do not copy
  collection data into presentation blocks (e.g. `cards`) by hand.

### Planned scholar features (on top of this architecture)
1. Import publications by DOI/ORCID (Crossref/OpenAlex) — a validated op + tool.
2. HĐGSNN points calculator — private to the owner; the owner may switch on a public
   badge ("Đủ điểm công trình chuẩn PGS – ngành X (tự đánh giá)"). Q1/Q2 counts, citations
   and h-index may be public.
3. Export the scientific CV (NAFOSTED / ministry template) to Word/PDF.

### Status (2026-11-14)
Steps 1–6 are built (blueprint v0.4.0, site v0.2.0, engine v0.2.0); not yet released or migrated.
1. Blueprint registry `ui/features/*.yaml` + `features.rs`, `widgets.rs`, `FeatureSetting.ai`,
   block/widget switches, tests (every tool once, every block/widget/tab kind owned, dangerous
   list unchanged, prompt parts placed).
2. Site reads it: prompt assembled from blueprint (test compares with the old prompt in
   `unit_tests/fixtures/old_prompt_template.txt`; documented differences in `prompt_tests.rs`),
   tools filtered and refused by feature + AI switch, `for_tool`/`for_path`/`DANGEROUS`/`RESERVED`
   from blueprint, widgets and blocks honour the switches. Master token runs every op.
3. Engine `/admin` plan board: "AI được dùng" per feature, "Xem prompt" viewer, block and widget
   switches; the site record carries `ai_features`, `disabled_blocks`, `disabled_widgets`.
4. Tab kinds Products, Community, Surveys, Associates (core blocks `product_index`,
   `community_index`, `survey_index`, `associate_index`); `set_special_page` removed (tabs have
   `show_in_nav` / `show_in_footer`). Fixed routes still answer for sites not yet migrated and hand
   over to the tab once it exists. **Migration not run yet** (the maintainer runs it by hand in
   DataGrip; migration scripts are not kept in the repos).
5. `/overview` (owner only) with its grouped menu (`OVERVIEW_MENU` in `staff/core.js`, also the
   "Chuyển ▾" switcher; the top bar no longer lists workspaces), "Quản lý trang" in the account
   menu, engine `/dashboard?site=N`, `User.default_site_id`, `PUT /api/platform/default-site`, `#N` on /account.
6. Scholar (`site.scholar`): `import_publications` (Crossref + OpenAlex), `sync_orcid` (awaiting
   review), `set_publication_scholar`, `set_scholar_settings`, private HĐGSNN self-assessment,
   optional public metrics/badge under the research tab, CV at `/overview/cv` (print → PDF) and
   `/overview/cv.doc` (Word). Needs api.crossref.org, api.openalex.org, pub.orcid.org reachable
   from the site. ORCID sync is on demand (no schedule yet).
Still open: page-view counts on the overview (GA4 only for now), per-block AI switches, a scheduled ORCID sync.

**In this repo:** add `ui/features/*.yaml` and the Rust that loads and validates it (like `ui/blocks`); extend `FeatureDef` and `FeatureSetting` (`ai: bool`, default true; switches for each Page kind, Block and Widget); add tab kinds Products, Community, Surveys, Associates with their core blocks; move the prompt text here; add `User.default_site_id: Option<String>`. Tests: every tool belongs to exactly one feature; every declared block, widget and tab kind exists.

### Interactive decks (`site.decks`, built 2026-11-14)
- Format: one YAML document per deck (`lmdeck v1`: `meta`, `theme`, `segments`, `data`, `slides`), validated by
  `learnmedi_blueprint::decks`. Slide kinds live in `learnmedi_blueprint/ui/slides/*.yaml` (same field schema as
  blocks, plus an `example`); a new kind = a YAML file there + its renderer in `learnmedi_site/crates/site/static/deck/player.js`.
  No HTML, CSS or free colours anywhere: theme = preset + checked hex colours + listed fonts + motion level.
- Storage: every version is a file `{prefix}/{site}/deck/{deck}/r{rev}.lmdeck.yaml` in the site's storage (never
  rewritten); Mongo `decks` keeps only the pointer, title, slug, published version and saved results. Undo restores the pointer.
- Ops (`decks::apply`): create_deck, set_deck, update_deck_meta, set_deck_theme, set_deck_data, add/update/move/duplicate/delete_slide,
  publish/unpublish_deck, reset_deck_results, delete_deck (dangerous). Reads: list_decks, get_deck, lint_deck, list_slide_kinds,
  get_slide_kind_schema, get_deck_theme_schema, get_deck_results. Every write returns `lint`.
- Workspace `/office` "Văn phòng" (staff page `static/staff/office.html` + `office.js`: visual editor, autosave, 5-step undo in the tab, sessions box, deck settings). Player routes: presenter
  `/office/presentation/{id}/present` (notes, clock per segment, hints, audience window, live session, QR card), draft
  `/__preview/deck/{id}`, published `/d/{slug}`, live audience `/live/{code}` (follows the presenter or browses freely; never
  sees notes, hints or hidden slides). Live sessions end on "Kết thúc", after 30 minutes without a heartbeat or after 12 hours;
  the presenter chooses whether the final state stays viewable (0 / 24 h / 7 days). Offline copy: `/api/decks/{id}/offline.html`.
- One-deck share link (`POST /api/decks/{id}/share-link`): 6 hours, reaches only `api_v1::DECK_SCOPE_READS/OPS` for that deck;
  its `/agent` page is `deck_guide.rs` (design brief: ask before guessing, pick kinds by intent, lint, preview).
- AI channel switches: every feature has "AI trợ lý" (sidebar, `FeatureSetting.ai`) and "AI ngoài" (share links / API,
  `FeatureSetting.share`), and the admin can take single tools away per channel (`PlanConfig.tools`). The site record carries
  `share_features`, `ai_blocked_tools`, `share_blocked_tools`; `features::tool_for_ai` / `tool_for_share` enforce them.
  Decks ship with the sidebar AI off and outside AI on.
