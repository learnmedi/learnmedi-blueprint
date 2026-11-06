//! 🚫 Guard: file bytes must never be stored in MongoDB.
//!
//! Engine and site store these models in Mongo. Files belong in Google Cloud
//! Storage; Mongo keeps only `Asset` metadata. If this fails, move the bytes to
//! GCS and store an `Asset` instead.

#[test]
fn models_have_no_binary_fields() {
    let src = include_str!("../models.rs");
    for bad in ["Vec<u8>", "Binary", "bytes::Bytes", "ByteBuf", "serde_bytes"] {
        assert!(!src.contains(bad), "models.rs contains `{bad}` — files go to GCS, not Mongo");
    }
}
