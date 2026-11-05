# Learnmedi Blueprint

Shared types and validation. Keep database queries, network calls, rendering and AI in the services.

Run `cargo build --locked` and `cargo test --locked` from this repository.

The internal API module defines shared request and response types. Bearer authentication runs in the engine service.

SiteSettings carries the values served by the engine. Its defaults let a site start while the engine is unavailable.
