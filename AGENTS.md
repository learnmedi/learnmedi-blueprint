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


## Repository scope

Shared types and validation. Keep database queries, network calls, rendering and AI in the services.
