# Feature architecture: change plan

Status: **steps 1–6 built 2026-11-14** (blueprint v0.4.0, site v0.2.0, engine v0.2.0), not yet released; the step 4 migration has not been run. See "Status" in AGENTS.md for what landed and what is still open.
Owner: maintainer of Learnmedi. Read together with the "Feature architecture" section of
`AGENTS.md` in `learnmedi_blueprint`, `learnmedi_site` and `learnmedi_engine` (same text in
all three; this file is the detailed version).

Repos (sibling folders under `RustroverProjects/learnmedi/`):
- `learnmedi_blueprint`: shared definitions, pure validation. No DB, HTTP, rendering, AI calls.
- `learnmedi_site`: every teacher site (`<slug>.learnmedi.com` + custom domains): public pages, Studio, AI site side.
- `learnmedi_engine`: accounts/login, site registry, payments, `/admin`, `/account`, `/dashboard` (host `engine.learnmedi.com`).
- `learnmedi_worker` (Python): runs the model loop; gets the prompt from the site via `/internal/ai/jobs/{id}/context`. **No change needed.**
- `learnmedi-home-fe`: marketing site. **No change.**

Hard rules from AGENTS.md still apply (no files in Mongo, AI edits only through `ops::apply`,
escape everything, secrets in `.env`, site → engine calls only, email consent, themes from references).

---

## 0. Why

Today the knowledge of "what a feature is" is scattered:

| Information | Where it is today |
|---|---|
| Feature list, tiers, admin plan board | `learnmedi_blueprint/src/plans.rs` (`CATALOG`, `FeatureDef`, `FeatureSetting`, `PlanConfig`) |
| Which route needs which feature | `learnmedi_site/crates/site/src/features.rs` `for_path` (hard-coded prefixes) |
| Which tool/op needs which feature | same file, `for_tool` — **matches tool names by substring** (`name.contains("post")` …). Fragile. |
| Tab kinds + core blocks | `learnmedi_blueprint/src/tabs.rs` (`TAB_KINDS`: Free, Blog, Courses, Collection, Link) |
| Fixed special pages | `learnmedi_site/crates/site/src/main.rs` + `public.rs`: `/community`, `/products`, `/products/{slug}`, `/research/surveys/{slug}`, `/associates`, `/learn` |
| Nav placement of `/community`, `/associates` | op/tool `set_special_page` (place: nav/footer/both/hidden) |
| Reserved slugs | `learnmedi_site/crates/site/src/ops.rs` line ~418 `RESERVED` |
| Dangerous tools (need confirm) | `learnmedi_site/crates/site/src/ai/guard.rs` `DANGEROUS` |
| AI tool definitions | `learnmedi_site/crates/site/src/ai/tools.rs` `definitions()` (~170 tools, one list) |
| AI prompt | `learnmedi_site/crates/site/src/ai/prompt.rs` `system_prompt()` — one long hand-written Vietnamese text + generated parts. Its first line hand-copies the tab kinds from `tabs.rs`. |
| Widgets | fortune button appended in `public.rs` ~line 257 when `fortune::button_on` (personality `envelope_on`); newsletter bottom-right prompt |

Goal: **one registry in blueprint** that declares every feature's skeleton and all AI prompt
text; site implements logic and reads the skeleton; engine gives the admin switches and a
prompt viewer. Adding a feature = one YAML file in blueprint + logic in site.

---

## 1. Decisions (final)

1. **Three visitor surface kinds**, nothing else:
   - **Page** = a tab kind with exactly one core block (Blog, Courses, Collection, and after the merge Products, Community, Surveys, Associates).
   - **Block** = a block type placed on pages. Blocks keep the existing 3 groups: `presentation` (typed-in content), `core` (main block of a special tab, only in its own tab), `excerpt` (reads data, allowed on any page incl. Free). Inside a group each block has its own `type` and fields; looks vary through select fields (`layout`, `frame`, `variant`, `image_ratio`, `style`…), the shared `style`, sections and the theme. No extra level below `type`.
   - **Widget** = site-wide element on every public page, switched on by the owner (fortune-stick button, newsletter prompt).
   - A feature may have several surfaces or none. Features visitors never see (Personality, Question bank, Classes…) only declare routes, tools, prompt.
   - No "other" bucket.
2. **All AI prompt text lives in blueprint** — the core/general part too. Site only assembles and injects runtime context. Engine reads blueprint to show prompts to the admin.
3. **Skeleton in blueprint, logic in site.**
4. **Admin board** (engine `/admin`): switch each feature, Page kind, Block and Widget on/off; switch **AI access** per item; **view the prompt text** of each item.
5. **Master token always runs every op and tool** (not limited by AI switches or features).
6. **Merge the fixed pages into tabs** (Products, Community, Surveys, Associates). `/learn` stays a fixed route (learner workspace).
7. Excerpt blocks can be added for any feature so its data can appear on a Free page (e.g. a future `community_list`), while its `*_index` core block stays in its own tab.
8. **Owner overview** at `{site_url}/overview` (owner only) + engine shortcut `/dashboard?site=N` (details §4).
9. Publications and achievements are **data** (collection items) shown by blocks; never hand-copy collection data into `cards` etc. (Today the Hồ Nguyên Như Ý demo home copies achievements into a `cards` block — that is the anti-pattern.)

Rejected along the way (do not reintroduce): putting workspace links in the top bar (grows forever); a separate "Danh hiệu & giải thưởng" menu item (it is collection content); `/dashboard` auto-redirecting by itself (use `?site=N`); `/dashboard/site/me` (ambiguous with 2 sites); storing the default site as a position.

---

## 2. Feature YAML (blueprint)

New folder `learnmedi_blueprint/ui/features/`, one file per feature, loaded with
`include_str!` like `ui/blocks/*.yaml` (see how `blocks.rs` builds `REGISTRY` from
`ui/blocks`). Proposed schema (adjust names if needed, keep the meaning):

```yaml
key: content.community          # EXISTING keys must not change (stored in PlanConfig + site.feature_overrides in Mongo)
label: Phục vụ cộng đồng
description: Trang /community kể lại những việc chủ site đã làm vì cộng đồng.
workspace: content              # studio | content | teaching | overview | settings
default_tier: standard          # basic | standard | premium
default_enabled: true
locked: false                   # true = always on, admin cannot switch off
surfaces:
  page:                         # omit when the feature has no Page
    tab_kind: community
    core_block: community_index
    allowed_block_groups: [presentation, core, excerpt]
    default_slug: community
    default_title: { vi: Phục vụ cộng đồng, en: Community service }
  blocks: [community_list]      # block types owned by this feature (core + excerpt)
  widgets: []                   # widget keys owned by this feature
routes:
  public: [/community, /en/community]
  staff: [/api/studio/community]
reserved_slugs: [community]
tools:                          # every AI tool is declared in exactly ONE feature file
  - name: get_community_contributions
  - name: set_community_settings
  - name: save_community_contribution
  - name: delete_community_contribution
    dangerous: true
prompt: |
  - Phục vụ cộng đồng: đọc get_community_contributions trước. …   (moved verbatim from prompt.rs)
```

Notes:
- **Tools that several features unlock**: today course/quiz tools need `TEACHING` *or* `SELF_PACED` (`features.rs` `COURSES`). Declare the tool once and add `also_enabled_by: [content.self_paced]` (or put them in a shared `courses` file whose `enabled_by: [teaching.classes, content.self_paced]`). Keep the "any one of" semantics.
- **Core file** `core.yaml` (key `studio.website`, `locked: true`): pages, blocks, sections, theme, navigation, translations, publish, undo, backups, theme ports/trials, intro, conversation tools, and the **general prompt** (role, how to work, find → show → confirm → act, multilingual rules, today's date placeholder, "one conversation = one task"). It also owns the placeholders the site fills: `{goal_block}`, `{focus}`, `{overview}`, `{catalog}`, `{theme}`, `{i18n_note}`, `{today}`, `{lang_note}`.
- Prompt text keeps the **current wording** (copy, do not rewrite) so behaviour does not change in step 2.
- Widgets: define a small list in blueprint, e.g. `widgets.rs` with `fortune_button`, `newsletter_prompt` (key, label, owner switch location, feature).

### Draft tool → feature map (verify against `features.rs::for_tool` and `features_tests.rs`)
Existing feature keys (plans.rs): `studio.website`, `studio.ai`, `studio.custom_domain`, `content.blog`, `content.newsletter`, `content.collections`, `content.community`, `commerce.products`, `content.self_paced`, `research.surveys`, `teaching.classes`.
New keys to add (all `locked: true` unless decided otherwise, so tiers do not change): `site.assets`, `site.personality`, `site.fortune`, `site.associates`.

- **studio.website (core)**: get_site_overview, get_page, get_page_snapshot, get_block_schema, list_page_templates, list_themes, get_section_schema, list_fonts, add_block, update_block, delete_block, move_block, duplicate_block, apply_page_template, set_page_sections, set_page_blocks, set_theme, update_site_settings, set_navigation, list_tabs, create_tab, update_tab, delete_tab (dangerous), create_page, update_page, delete_page (dangerous), publish_page, unpublish_page, publish_all, set_translation, mark_translation_synced, list_revisions, undo, preview_theme_port, apply_theme_port, suggest_similar_themes, create_theme_trial, publish_theme_trial, discard_theme_trial, list_site_backups, restore_site_backup, set_intro, set_conversation_goal, finish_conversation, flag_out_of_scope, set_special_page (remove after the step 4 merge)
- **site.assets**: list_assets, get_storage_usage, move_assets, rename_asset, delete_assets (dangerous), import_image_url, get_video_info, read_document
- **content.blog**: list_posts, get_post, create_post, update_post, publish_post, unpublish_post, delete_post (dangerous), list_post_comments, moderate_comment
- **content.collections**: list_collection_items, add_collection_items, update_collection_item, delete_collection_items (dangerous), publish_collection_item, set_collection_featured_items
- **content.community**: get_community_contributions, set_community_settings, save_community_contribution, delete_community_contribution (dangerous)
- **research.surveys**: list_research_surveys, save_research_survey
- **commerce.products**: list_products, save_product, delete_product
- **courses (teaching.classes OR content.self_paced)**: list_courses, get_course, create_course, update_course, add_course_week, add_course_item, update_course_item, delete_course_item, publish_course, list_quizzes, get_quiz, get_quiz_results, create_quiz, update_quiz, add_quiz_questions, update_quiz_question, delete_quiz_question, publish_quiz, delete_quiz (dangerous)
- **teaching.classes**: share_quiz, copy_class_content, get_attempt, export_gradebook, search_question_bank, list_bank_tags, bank_add_questions, bank_update_question, bank_delete_questions (dangerous), bank_save_from_quiz, create_quiz_from_bank, add_bank_questions_to_quiz, list_classrooms, get_class_report, get_course_report, list_submissions, grade_submission, grade_quiz_answer, create_classroom, update_classroom, add_class_members, remove_class_member (dangerous), post_class_announcement
- **content.newsletter**: get_newsletter_stats, list_subscribers, get_newsletter, create_newsletter, update_newsletter, delete_newsletter (dangerous), send_newsletter_test, send_newsletter (dangerous), add_subscribers, remove_subscribers (dangerous)
- **site.personality**: set_personality
- **site.fortune**: get_fortune_brief, set_fortune_quotes (+ widget `fortune_button`)
- **site.associates**: list_associates, add_associate, remove_associate
- **studio.ai**: gate for the assistant itself (`/api/ai`), no tools of its own

The dangerous list must equal today's `guard.rs::DANGEROUS`: delete_post, delete_page, delete_tab, delete_assets, delete_quiz, delete_collection_items, delete_community_contribution, bank_delete_questions, remove_class_member, remove_subscribers, delete_newsletter, send_newsletter.

### Settings types (blueprint `plans.rs`)
- `FeatureDef` is built from the YAML (or keeps its const fields and the YAML adds the rest — choose one source; YAML preferred).
- `FeatureSetting` gains `#[serde(default = "yes")] ai: bool` → old stored `PlanConfig` documents mean "AI on" (no migration).
- Switches for Page kinds, Blocks, Widgets: add to `PlanConfig` e.g. `blocks: BTreeMap<String, bool>`, `widgets: BTreeMap<String, bool>`, `ai_blocks…` only if needed; default = on. A Page kind is switched with its feature. Keep it minimal: switch per feature, per block type, per widget; AI per feature (and per block type if the admin should hide a block from the AI).
- Effective view sent to the site with the record (today `site.features: Option<Vec<String>>` is computed in engine `plans.rs` line ~41 `cfg.resolve(...)`): add `ai_features`, `disabled_blocks`, `disabled_widgets` (all `#[serde(default, skip_serializing_if…)]`, `None` = old engine = allow all, same rule as `features`).

### Prompt functions (blueprint)
- `features::prompt_for(enabled_for_ai: &[&str]) -> String` — core text + each enabled feature's `prompt`, in a fixed order.
- `features::prompt_parts()` — every item's text for the admin viewer.
- Keep `blocks::catalog_for_prompt()`; filter out disabled block types (take a list argument).
- Replace the hand-written tab-kinds line by a generated `tabs_for_prompt()`.

### Tests (blueprint)
- Every YAML parses; keys unique; every existing `plans` key present with unchanged key string.
- Every `surfaces.blocks` entry exists in the block registry; every `core_block` exists and is group `core`; every widget exists.
- Every tool name declared once across all files.
- `reserved_slugs` ∪ system routes has no duplicates with tab default slugs of other features.
- Dangerous list equals the old `DANGEROUS` (snapshot test until step 2 lands).

---

## 3. Step-by-step

Each step ends green (`cargo test` in the touched repos). Blueprint changes need a version
bump + tag, then engine **and** site set the new tag (see AGENTS.md "learnmedi_blueprint
dependency and releases"; locally `.cargo/config.toml` patches to `../learnmedi_blueprint`).

### Step 1 — Blueprint registry (no behaviour change)
Files:
- `ui/features/*.yaml` (new): core, blog, courses, collections, community, surveys, products, self_paced (may merge with courses), teaching, newsletter, assets, personality, fortune, associates, ai, custom_domain.
- `src/features.rs` (new): load/validate, `tool_feature(name)`, `dangerous_tools()`, `reserved_slugs()`, `routes()`, `prompt_for()`, `prompt_parts()`, `tabs_for_prompt()`.
- `src/plans.rs`: wire `CATALOG`/`FeatureDef` to YAML; `FeatureSetting.ai`; block/widget switches; new locked keys.
- `src/widgets.rs` (new, small) or inside features.rs.
- `src/lib.rs`: export modules.
- `docs/architecture.md`: describe features.rs, widgets, prompt.
- Tests listed in §2.
Do **not** add the new tab kinds yet (step 4).

### Step 2 — Site reads blueprint (behaviour identical)
Files in `learnmedi_site/crates/site/src`:
- `ai/prompt.rs`: build the prompt from `learnmedi_blueprint::features::prompt_for(...)` + runtime context (`site_overview`, `goal_block`, focus, today, lang note, i18n note, catalog, theme). Delete the hand-written prose. **Acceptance:** for a site with every feature on, the new prompt equals the old one (allow whitespace/order differences you document). Write a test that renders both once (keep old text in a test fixture until merged).
- `ai/tools.rs`: `definitions()` filtered by the site's features **and** AI switches. Add a runtime check where a tool call is executed (the AI job path in `ai/jobs.rs` → tool executor): refuse a tool whose feature is off or AI-disabled, with a clear Vietnamese error. Master token / `/api/v1/ops` are not affected.
- `features.rs`: `for_tool` and `for_path` read the blueprint map instead of substring matching. Keep `has`, `has_any`, `require`, `guard`.
- `ai/guard.rs`: `DANGEROUS` from blueprint.
- `ops.rs`: `RESERVED` from blueprint (system routes + feature reserved slugs + `overview`).
- `public.rs`: widgets render only when the owner switch AND the admin switch are on.
- Studio (`static/studio.js`): hide block types and sidebar tabs that are switched off (read from bootstrap).
- Tests: `tools_tests.rs` (every tool has a feature; disabled feature → tool not listed and refused), `features_tests.rs` (route map unchanged), `every_write_op_has_a_tool` stays.

### Step 3 — Engine admin board
Files in `learnmedi_engine/crates/engine`:
- `src/plans.rs`: resolve and send the new effective fields with the site record.
- `src/admin.rs` + `/api/admin/plans` (`plans::admin_get/admin_put`): include block/widget switches, AI switches, and `prompt_parts()` for the viewer.
- `static/admin.js` (plan board, around line 38–60): per feature card add an **"AI được dùng"** toggle and a **"Xem prompt"** button (modal, read-only, shows that feature's prompt text and its tools, dangerous ones marked); add sections for Blocks and Widgets with on/off toggles.
- Master token unchanged.

### Step 4 — Merge fixed pages into tab kinds (one at a time: products → community → surveys → associates)
Per feature:
- Blueprint: add `TabKind::<X>` to `tabs.rs` + `TAB_KINDS` entry; add core block YAML `ui/blocks/NN-<x>_index.yaml` (group `core`); YAML `surfaces.page`. Bump tag.
- Site: render the core block in `render.rs`; the tab is served by the normal `/{slug}` path; keep detail routes (`/products/{slug}`, `/research/surveys/{slug}`) as they are or map them to `/{tab}/{item}` — keep old URLs working. Remove the fixed route only after the tab serves it. `community` / `products` leave `RESERVED` for the tab kind that owns them (only that kind may take the slug).
- `set_special_page`: after community + associates are tabs, nav placement comes from the tab's nav settings; migrate the stored placement (nav/footer/both/hidden) into the tab and remove the op/tool (update `WRITE_OPS`, `SHARE_OPS` lists in `ops.rs` and tools).
- **Migration script** (give to maintainer, they run it on Mongo): for every site that uses the feature (has data in its collection, or has the special page placed in nav/footer), insert one page document of the new tab kind with the old slug, the core block, the stored nav placement, `nav_order` at the end; skip sites that already have a tab of that kind (idempotent); write a `Revision` if the script runs through the app, or document that a direct Mongo insert has no undo. Data collections are **not** moved.
- AI prompt line for that feature updated in its YAML.

### Step 5 — Owner overview + dashboard shortcut + default site (§4)

### Step 6 — Scholar features (§5)

---

## 4. Owner overview and dashboard shortcut (spec)

### 4.1 `{site_url}/overview` (site repo)
- Route `/overview` in `main.rs` → `studio::workspace_page(req, st, "overview")`. `workspace_page` today allows any staff; add: workspace `overview` requires **owner** (`v.role_str() == "owner"`), editors get the "Không có quyền" page.
- `overview` added to reserved slugs. (No existing page uses it — confirmed by maintainer.)
- Same staff shell `static/studio.html` / `studio.js`. When `workspace === "overview"`: own grouped left sidebar; hide top-bar actions that do not apply (Hoàn tác, Xuất bản, Chia sẻ link, AI).
- Sidebar (defined once as a data list, also used by the "Chuyển ▾" switcher):
  - Tổng quan
  - **Hồ sơ khoa học**: Công bố · Chỉ số · Điểm HĐGSNN · Lý lịch khoa học
  - **Website**: Studio ↗ (`/studio`) · Nội dung ↗ (`/content`) · Lượt xem
  - Giảng dạy ↗ (`/teach`, only when the site has `teaching.classes`)
  - Cài đặt ↗ (`/settings`)
  (↗ = navigates to the existing workspace page; others render inside overview.) Later: the list comes from blueprint (features with `workspace: overview`).
- Main panel "Tổng quan" shows: stat cards (publications, Q1/Q2 count, citations, h-index, views last 30 days — start with `SiteStats` counters that exist: pages, posts, courses, students, subscribers…, add the scholar numbers in step 6), HĐGSNN progress bar vs PGS/GS standard (step 6), publications found via ORCID awaiting review (step 6), to-do list (missing portrait, papers without DOI, no English version…).
- In Studio/Content/Teaching/Settings the top bar **no longer lists workspaces** (`ws-nav` in `studio.html` lines ~21–26): replace with a link **"Quản lý trang"** → `/overview` (owner only; editors keep a way to switch workspaces via the "Chuyển ▾" menu) and the "Chuyển ▾" switcher.
- Studio keeps its own inner sidebar (Các tab của site, Bài viết, …) unchanged.
- The public-site account menu (`render.rs` ~line 1238, `acct-ws` grid Studio/Nội dung/Giảng dạy/Cài đặt) gets "Quản lý trang" first for owners.

### 4.2 Engine `/dashboard?site=N`
Today `platform::dashboard_page` only redirects to `/account#sites`; the real account page is `/account` (`static/dashboard.html`, `dashboard.js`). Keep that when there is no `site` param.
- Extract `accessible_sites(st, &u)` from `platform::my_sites` (query: owner_id = user, or staff.user_id = user, or staff.email = user email; master user = all). **Order by `ts_created` ascending** for numbering. `my_sites` (used by `/account`) switches to the same ascending order and `dashboard.js` shows `#1`, `#2` on each site card.
- `GET /dashboard?site=N`:
  - not signed in → `/login?next=/dashboard%3Fsite%3DN` (the param is `next`)
  - `N = 0` → `User.default_site_id` if it is still accessible, else the first site
  - `N ≥ 1` → the N-th site
  - non-number, out of range, no sites → `/account#sites`
  - valid → `302 {site_public_url(site)}/overview` (`site_public_url` already prefers a verified custom domain)
- Admins: `?site=` works on their own sites; the all-sites list stays `/admin`.
- Default site: blueprint `User` gains `#[serde(default)] pub default_site_id: Option<String>` (store the id, never a position). `PUT /api/platform/default-site {site_id}` checks access and saves; `my_sites` returns `default_site_id`; `dashboard.js` adds a ⭐ "Đặt làm mặc định" button per card.
- Emails to owners always link `https://engine.learnmedi.com/dashboard?site=0`.
- Tests: resolver cases (0 with/without default, default deleted, N in range, out of range, `abc`, editor-only sites, signed out).

---

## 5. Scholar features (after steps 1–5)

Each = one feature YAML in blueprint (surfaces, tools, prompt) + ops/tools/UI in site. AI writes go through new validated ops in `ops.rs` + tools in `ai/tools.rs` (AGENTS rule 2; `every_write_op_has_a_tool`).

1. **Import by DOI/ORCID** (first; it feeds 2 and 3): paste DOIs or an ORCID iD → fetch metadata from Crossref/OpenAlex (title, authors, venue, year, DOI, citations) → add items to the site's "Nghiên cứu" collection (template `research` already has `authors, year, venue, doi, link_url, tags, summary, body_md`). ORCID sync lists new works as "awaiting review" on the overview. Network calls happen in the site (or worker) — not in blueprint. Check the egress allowlist for api.crossref.org / api.openalex.org / pub.orcid.org.
2. **HĐGSNN points**: per-paper points + total vs the PGS/GS standard of the chosen field, journal rank Q1–Q4. **Private to the owner** by default. The owner may switch on a public badge "Đủ điểm công trình chuẩn PGS – ngành X (tự đánh giá)". Public numbers allowed: Q1/Q2 count, citations, h-index. Label everything as self-assessed.
3. **Scientific CV export**: Word/PDF in the NAFOSTED / ministry template from the profile data.

---

## 6. Release order

1. Blueprint step 1 → bump version, merge to `release` (tags `vX.Y.Z`) → set tag in engine and site → `cargo build` → commit `Cargo.toml` + `Cargo.lock`.
2. Site step 2 (+ engine passes the new effective fields: engine must ship before or with the site; the site must treat missing fields as "allow all").
3. Engine step 3.
4. Step 4 per feature: blueprint tag → site → migration script run by maintainer → remove the fixed route.
5. Step 5: blueprint (`default_site_id`) → engine → site.
6. Step 6 features.

Bump `version` in `crates/<crate>/Cargo.toml` for every release (an existing tag fails the pipeline). CI runs only on the `release` branch.

---

## 7. Open points to settle while building

- YAML as the single source for `FeatureDef` vs keeping the Rust consts — prefer YAML; keep key strings identical.
- Whether AI switches are per feature only or also per block type (start per feature; add per block if the admin needs it).
- Detail URLs after the merge (`/products/{slug}` vs `/{tab}/{item}`): keep old URLs working either way.
- Editors (staff role `editor`) have no `/overview`; confirm how they switch workspaces once `ws-nav` is gone ("Chuyển ▾").
- Where ORCID sync runs on a schedule (site background task vs `learnmedi_worker`).

## 8. Other context from the planning session (not code)
- Outreach: cold email template to Vietnamese researchers saved at `C:\Users\Admin\Downloads\learnmedi-email-template.md` (links only, no client names). Target list: researchers recently in the press (Stanford/Elsevier top 2% 2025, Quả Cầu Vàng 2025, Gương mặt trẻ 2025) without a personal site.
- The three existing demo sites (`learnmedi_site/scripts/demo/`) are lecturers of Trường Quốc tế – ĐHQGHN (VNU-IS): Đào Công Tuấn, Hồ Nguyên Như Ý, Trần Thị Hoàng Yến.
